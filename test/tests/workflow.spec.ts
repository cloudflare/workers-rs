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
      await new Promise((resolve) => setTimeout(resolve, 500));
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
      await new Promise((resolve) => setTimeout(resolve, 500));
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
    }

    expect(status).toBe("Errored");
  });
});
