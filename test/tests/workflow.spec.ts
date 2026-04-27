import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

describe("workflow", () => {
  test("create and poll status until completion", async () => {
    const createResp = await mf.dispatchFetch(`${mfUrl}workflow/create`, {
      method: "POST",
    });
    expect(createResp.status).toBe(200);
    const { id } = (await createResp.json()) as { id: string };
    expect(id).toBeDefined();
    expect(typeof id).toBe("string");

    let status: string | undefined;
    let output: unknown;
    for (let i = 0; i < 30; i++) {
      const statusResp = await mf.dispatchFetch(
        `${mfUrl}workflow/status/${id}`
      );
      expect(statusResp.status).toBe(200);
      const body = (await statusResp.json()) as {
        status: string;
        output: unknown;
        error: unknown;
      };
      status = body.status;
      output = body.output;
      if (status === "Complete" || status === "Errored") {
        break;
      }
      await new Promise((resolve) => setTimeout(resolve, 500));
    }

    expect(status).toBe("Complete");
    expect(output).toEqual({
      processed: "hello",
      validation: { valid: true, attempt: 1 },
    });
  });

  test("non-retryable error stops workflow immediately", async () => {
    const createResp = await mf.dispatchFetch(
      `${mfUrl}workflow/create-invalid`,
      { method: "POST" }
    );
    expect(createResp.status).toBe(200);
    const { id } = (await createResp.json()) as { id: string };
    expect(id).toBeDefined();

    let status: string | undefined;
    for (let i = 0; i < 30; i++) {
      const statusResp = await mf.dispatchFetch(
        `${mfUrl}workflow/status/${id}`
      );
      expect(statusResp.status).toBe(200);
      const body = (await statusResp.json()) as {
        status: string;
        output: unknown;
      };
      status = body.status;
      if (status === "Complete" || status === "Errored") {
        break;
      }
      await new Promise((resolve) => setTimeout(resolve, 500));
    }

    expect(status).toBe("Errored");
  });

  async function lifecycleStatus(
    id: string
  ): Promise<{ status: string; error: unknown }> {
    const resp = await mf.dispatchFetch(
      `${mfUrl}workflow/lifecycle/status/${id}`
    );
    return (await resp.json()) as { status: string; error: unknown };
  }

  async function pollUntil(
    id: string,
    predicate: (status: string) => boolean
  ): Promise<string> {
    for (let i = 0; i < 10; i++) {
      const { status } = await lifecycleStatus(id);
      if (predicate(status)) return status;
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
    const { status } = await lifecycleStatus(id);
    return status;
  }

  async function createLifecycleWorkflow(): Promise<string> {
    const resp = await mf.dispatchFetch(
      `${mfUrl}workflow/lifecycle/create`,
      { method: "POST" }
    );
    expect(resp.status).toBe(200);
    const { id } = (await resp.json()) as { id: string };
    await pollUntil(id, (s) => s !== "Queued");
    return id;
  }

  async function lifecycleAction(
    action: string,
    id: string
  ): Promise<Response> {
    return mf.dispatchFetch(
      `${mfUrl}workflow/lifecycle/${action}/${id}`,
      { method: "POST" }
    );
  }

  test("pause and resume a running workflow", async () => {
    const id = await createLifecycleWorkflow();

    expect((await lifecycleAction("pause", id)).status).toBe(200);
    const paused = await pollUntil(id, (s) => s === "Paused");
    expect(paused).toBe("Paused");

    expect((await lifecycleAction("resume", id)).status).toBe(200);
    const resumed = await pollUntil(id, (s) => s !== "Paused");
    expect(resumed).not.toBe("Paused");

    await lifecycleAction("terminate", id);
  });

  test("terminate a running workflow", async () => {
    const id = await createLifecycleWorkflow();

    expect((await lifecycleAction("terminate", id)).status).toBe(200);
    const status = await pollUntil(id, (s) => s === "Terminated");
    expect(status).toBe("Terminated");
  });

  test("restart a running workflow", async () => {
    const id = await createLifecycleWorkflow();

    expect((await lifecycleAction("restart", id)).status).toBe(200);
    const status = await pollUntil(
      id,
      (s) => s !== "Queued"
    );
    expect(["Running", "Waiting", "Queued"]).toContain(status);

    await lifecycleAction("terminate", id);
  });

  test("wait_for_event receives sent event", async () => {
    const createResp = await mf.dispatchFetch(
      `${mfUrl}workflow/event/create`,
      { method: "POST" }
    );
    expect(createResp.status).toBe(200);
    const { id } = (await createResp.json()) as { id: string };
    expect(id).toBeDefined();

    // Give the workflow time to reach wait_for_event
    await new Promise((resolve) => setTimeout(resolve, 1000));

    // Send the event
    const sendResp = await mf.dispatchFetch(
      `${mfUrl}workflow/event/send/${id}`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ approved: true, reason: "looks good" }),
      }
    );
    expect(sendResp.status).toBe(200);

    // Poll until complete
    let status: string | undefined;
    let output: any;
    for (let i = 0; i < 30; i++) {
      const statusResp = await mf.dispatchFetch(
        `${mfUrl}workflow/event/status/${id}`
      );
      expect(statusResp.status).toBe(200);
      const body = (await statusResp.json()) as {
        status: string;
        output: any;
      };
      status = body.status;
      output = body.output;
      if (status === "Complete" || status === "Errored") {
        break;
      }
      await new Promise((resolve) => setTimeout(resolve, 500));
    }

    expect(status).toBe("Complete");
    expect(output).toEqual({
      approved: true,
      reason: "looks good",
      type: "approval",
    });
  });
});
