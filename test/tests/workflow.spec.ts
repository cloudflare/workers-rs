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
    expect(output).toEqual({ processed: "hello" });
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
    expect(output.payload).toEqual({ approved: true, reason: "looks good" });
    expect(output.event_type).toBe("approval");
  });
});
