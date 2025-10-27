import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

describe("Panic Hook with WASM Reinitialization", () => {
  // These tests are explicitly run sequentially with a longer timeout
  // to ensure they fully run the reinitialization lifecycle.
  test("panic recovery tests", async () => {
    // basic panic recovery
    {
      await mf.dispatchFetch(`${mfUrl}durable/COUNTER`);
      const resp = await mf.dispatchFetch(`${mfUrl}durable/COUNTER`);
      expect(await resp.text()).toContain("unstored_count: 2");

      const panicResp = await mf.dispatchFetch(`${mfUrl}test-panic`);
      expect(panicResp.status).toBe(500);

      const panicText = await panicResp.text();
      expect(panicText).toContain("Workers runtime canceled");

      const normalResp = await mf.dispatchFetch(`${mfUrl}durable/COUNTER`);
      expect(await normalResp.text()).toContain("unstored_count: 1");
    }

    // multiple requests after panic all succeed
    {
      const panicResp = await mf.dispatchFetch(`${mfUrl}test-panic`);
      expect(panicResp.status).toBe(500);

      const requests = [
        mf.dispatchFetch(`${mfUrl}durable/hello`),
        mf.dispatchFetch(`${mfUrl}durable/hello`),
        mf.dispatchFetch(`${mfUrl}durable/hello`),
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
        mf.dispatchFetch(`${mfUrl}test-panic`), // This will panic
        mf.dispatchFetch(`${mfUrl}durable/hello`), // This should succeed after reinitialization
        mf.dispatchFetch(`${mfUrl}durable/hello`),
      ];

      const responses = await Promise.all(simultaneousRequests);

      // should always have one error and one ok
      let foundErrors = 0;
      for (const response of responses) {
        if (response.status === 500) {
          expect(foundErrors).toBeLessThan(2);
          foundErrors++;
        } else {
          expect(response.status).toBe(200);
        }
      }
      expect(foundErrors).toBeGreaterThan(0);
    }

    // worker continues to function normally after multiple panics
    {
      for (let cycle = 1; cycle <= 3; cycle++) {
        const panicResp = await mf.dispatchFetch(`${mfUrl}test-panic`);
        expect(panicResp.status).toBe(500);

        const recoveryResp = await mf.dispatchFetch(`${mfUrl}durable/hello`);
        expect(recoveryResp.status).toBe(200);
      }
    }

    // explicit abort() recovery test
    {
      await mf.dispatchFetch(`${mfUrl}durable/COUNTER`);
      const resp = await mf.dispatchFetch(`${mfUrl}durable/COUNTER`);
      expect(await resp.text()).toContain("unstored_count:");

      const abortResp = await mf.dispatchFetch(`${mfUrl}test-abort`);
      expect(abortResp.status).toBe(500);

      const abortText = await abortResp.text();
      expect(abortText).toContain("Workers runtime canceled");

      const normalResp = await mf.dispatchFetch(`${mfUrl}durable/COUNTER`);
      expect(await normalResp.text()).toContain("unstored_count: 1");
    }

    // out of memory recovery test
    {
      await mf.dispatchFetch(`${mfUrl}durable/COUNTER`);
      const resp = await mf.dispatchFetch(`${mfUrl}durable/COUNTER`);
      expect(await resp.text()).toContain("unstored_count:");

      const oomResp = await mf.dispatchFetch(`${mfUrl}test-oom`);
      expect(oomResp.status).toBe(500);

      const oomText = await oomResp.text();
      expect(oomText).toContain("Workers runtime canceled");

      const normalResp = await mf.dispatchFetch(`${mfUrl}durable/COUNTER`);
      expect(await normalResp.text()).toContain("unstored_count: 1");
    }

    // JS error recovery test
    // TODO: figure out how to achieve this one. Hard part is global error handler
    // will need to detect JS errors, not just WebAssembly.RuntimeError, which
    // may over-classify.
    // {
    //   await mf.dispatchFetch(`${mfUrl}durable/COUNTER`);
    //   const resp = await mf.dispatchFetch(`${mfUrl}durable/COUNTER`);
    //   expect(await resp.text()).toContain("unstored_count:");

    //   const jsErrorResp = await mf.dispatchFetch(`${mfUrl}test-js-error`);
    //   expect(jsErrorResp.status).toBe(500);

    //   const jsErrorText = await jsErrorResp.text();
    //   expect(jsErrorText).toContain("Workers runtime canceled");

    //   const normalResp = await mf.dispatchFetch(`${mfUrl}durable/COUNTER`);
    //   expect(await normalResp.text()).toContain("unstored_count: 1");
    // }
  }, 20_000);
});
