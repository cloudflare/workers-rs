import { describe, test, expect } from "vitest";
import { mf } from "./mf";

describe("Panic Hook with WASM Reinitialization", () => {
  test("panic recovery tests", async () => {
    // basic panic recovery
    {
      const panicResp = await mf.dispatchFetch("http://fake.host/test-panic");
      expect(panicResp.status).toBe(500);

      const panicText = await panicResp.text();
      expect(panicText).toContain("Workers runtime canceled");

      const normalResp = await mf.dispatchFetch("http://fake.host/durable/hello");
      expect(normalResp.status).toBe(200);

      const normalText = await normalResp.text();
      expect(normalText).toContain("Hello from my-durable-object!");
    }

    // multiple requests after panic all succeed
    {
      const panicResp = await mf.dispatchFetch("http://fake.host/test-panic");
      expect(panicResp.status).toBe(500);

      const requests = [
        mf.dispatchFetch("http://fake.host/durable/hello"),
        mf.dispatchFetch("http://fake.host/durable/hello"),
        mf.dispatchFetch("http://fake.host/durable/hello"),
      ];

      const responses = await Promise.all(requests);

      for (let i = 0; i < responses.length; i++) {
        const text = await responses[i].text();
        expect(responses[i].status).toBe(200);
        expect(text).toContain("Hello from my-durable-object!");
      }
    }

    // simultaneous requests during panic handling
    {
      const simultaneousRequests = [
        mf.dispatchFetch("http://fake.host/test-panic"), // This will panic
        mf.dispatchFetch("http://fake.host/durable/hello"), // This should succeed after reinitialization
        mf.dispatchFetch("http://fake.host/durable/hello"),
      ];

      const responses = await Promise.all(simultaneousRequests);

      expect(responses[0].status).toBe(500);

      for (let i = 1; i < responses.length; i++) {
        expect(responses[i].status).toBe(200);
      }
    }

    // worker continues to function normally after multiple panics
    {
      for (let cycle = 1; cycle <= 3; cycle++) {
        const panicResp = await mf.dispatchFetch("http://fake.host/test-panic");
        expect(panicResp.status).toBe(500);

        const recoveryResp = await mf.dispatchFetch(
          "http://fake.host/durable/hello"
        );
        expect(recoveryResp.status).toBe(200);
      }
    }
  });
});
