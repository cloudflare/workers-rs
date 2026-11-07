import { describe, test, expect, vi } from "vitest";
import { mf, mfUrl } from "./mf";

type InstanceStatus = { status: string; output: unknown; error: unknown };

const settled = (status: string) =>
  status === "Complete" || status === "Errored";

async function create(path: string): Promise<string> {
  const resp = await mf.dispatchFetch(`${mfUrl}workflow/${path}`, {
    method: "POST",
  });
  expect(resp.status).toBe(200);
  const { id } = (await resp.json()) as { id: string };
  expect(typeof id).toBe("string");
  return id;
}

// Polls the instance's status until `done` accepts it.
function waitForStatus(
  kind: string,
  id: string,
  done: (status: string) => boolean
): Promise<InstanceStatus> {
  return vi.waitFor(
    async () => {
      const resp = await mf.dispatchFetch(
        `${mfUrl}workflow/${kind}/status/${id}`
      );
      expect(resp.status).toBe(200);
      const body = (await resp.json()) as InstanceStatus;
      expect(done(body.status), `status is ${body.status}`).toBe(true);
      return body;
    },
    { timeout: 15_000, interval: 100 }
  );
}

async function lifecycleAction(action: string, id: string) {
  const resp = await mf.dispatchFetch(
    `${mfUrl}workflow/lifecycle/${action}/${id}`,
    { method: "POST" }
  );
  expect(resp.status).toBe(200);
}

async function createRunningLifecycleWorkflow(): Promise<string> {
  const id = await create("lifecycle/create");
  await waitForStatus("lifecycle", id, (s) => s !== "Queued");
  return id;
}

describe("workflow", () => {
  test("create and poll status until completion", async () => {
    const id = await create("test/create");
    const { status, output } = await waitForStatus("test", id, settled);
    expect(status).toBe("Complete");
    expect(output).toEqual({
      processed: "hello",
      validation: { valid: true, attempt: 1 },
    });
  });

  test("non-retryable error stops workflow immediately", async () => {
    const id = await create("test/create-invalid");
    const { status } = await waitForStatus("test", id, settled);
    expect(status).toBe("Errored");
  });

  test("pause and resume a running workflow", async () => {
    const id = await createRunningLifecycleWorkflow();

    await lifecycleAction("pause", id);
    await waitForStatus("lifecycle", id, (s) => s === "Paused");

    await lifecycleAction("resume", id);
    await waitForStatus("lifecycle", id, (s) => s !== "Paused");

    await lifecycleAction("terminate", id);
  });

  test("terminate a running workflow", async () => {
    const id = await createRunningLifecycleWorkflow();

    await lifecycleAction("terminate", id);
    await waitForStatus("lifecycle", id, (s) => s === "Terminated");
  });

  test.each(["restart", "restart-from"])(
    "%s a running workflow",
    async (action) => {
      const id = await createRunningLifecycleWorkflow();

      await lifecycleAction(action, id);
      const { status } = await waitForStatus(
        "lifecycle",
        id,
        (s) => s !== "Queued"
      );
      expect(["Running", "Waiting"]).toContain(status);

      await lifecycleAction("terminate", id);
    }
  );

  test("wait_for_event receives sent event", async () => {
    const id = await create("event/create");
    // The engine can't take events until it has started the instance.
    await waitForStatus("event", id, (s) => s !== "Queued");

    const sendResp = await mf.dispatchFetch(
      `${mfUrl}workflow/event/send/${id}`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ approved: true, reason: "looks good" }),
      }
    );
    expect(sendResp.status).toBe(200);

    const { status, output } = await waitForStatus("event", id, settled);
    expect(status).toBe("Complete");
    expect(output).toEqual({
      approved: true,
      reason: "looks good",
      type: "approval",
    });
  });

  test("saga rollback runs when a downstream step fails", async () => {
    const id = await create("rollback/create");

    // The second step fails non-retryably, so the instance ends up errored.
    const { status } = await waitForStatus("rollback", id, settled);
    expect(status).toBe("Errored");

    // The rollback handler records the error that triggered it in KV.
    const marker = async () => {
      const resp = await mf.dispatchFetch(
        `${mfUrl}workflow/rollback/marker/${id}`
      );
      return ((await resp.json()) as { marker: string | null }).marker;
    };
    await expect.poll(marker, { timeout: 15_000 }).toContain("kaboom");
  });
});
