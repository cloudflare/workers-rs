import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

describe("Panic Hook with WASM Reinitialization", () => {
  // These tests are explicitly run sequentially with a longer timeout
  // to ensure they fully run the reinitialization lifecycle.
  test("panic recovery tests", async () => {
    // First, detect which panic mode we're running in by checking the error message
    const detectResp = await mf.dispatchFetch(`${mfUrl}test-panic`);
    const detectText = await detectResp.text();
    
    // panic=unwind mode returns "PanicError:" in the response
    // panic=abort mode returns "Workers runtime canceled"
    const isPanicUnwind = detectText.includes("PanicError:");
    
    if (isPanicUnwind) {
      // ===== PANIC=UNWIND MODE TESTS =====
      // In this mode, panics are caught and converted to JS errors.
      // The Worker continues without reinitialization.
      
      // Test 1: Basic panic recovery - counter should NOT reset after panic
      // We use a unique durable object ID to get fresh state
      {
        const uniqueId = `UNWIND_${Date.now()}_${Math.random()}`;
        
        // Call the durable object twice to establish a counter
        const resp1 = await mf.dispatchFetch(`${mfUrl}durable/${uniqueId}`);
        const text1 = await resp1.text();
        // Extract the unstored_count from the response
        const match1 = text1.match(/unstored_count: (\d+)/);
        const count1 = match1 ? parseInt(match1[1]) : 0;

        const resp2 = await mf.dispatchFetch(`${mfUrl}durable/${uniqueId}`);
        const text2 = await resp2.text();
        const match2 = text2.match(/unstored_count: (\d+)/);
        const count2 = match2 ? parseInt(match2[1]) : 0;
        expect(count2).toBe(count1 + 1);

        // Now trigger a panic
        const panicResp = await mf.dispatchFetch(`${mfUrl}test-panic`);
        expect(panicResp.status).toBe(500);
        const panicText = await panicResp.text();
        expect(panicText).toContain("PanicError:");
        expect(panicText).toContain("Intentional panic");

        // Counter should continue from where it was (not reset)
        const resp3 = await mf.dispatchFetch(`${mfUrl}durable/${uniqueId}`);
        const text3 = await resp3.text();
        const match3 = text3.match(/unstored_count: (\d+)/);
        const count3 = match3 ? parseInt(match3[1]) : 0;
        expect(count3).toBe(count2 + 1);
      }

      // Test 2: Multiple panics don't affect subsequent requests
      {
        const uniqueId = `UNWIND2_${Date.now()}_${Math.random()}`;
        
        const resp1 = await mf.dispatchFetch(`${mfUrl}durable/${uniqueId}`);
        const match1 = (await resp1.text()).match(/unstored_count: (\d+)/);
        const count1 = match1 ? parseInt(match1[1]) : 0;

        // Trigger multiple panics
        for (let i = 0; i < 3; i++) {
          const panicResp = await mf.dispatchFetch(`${mfUrl}test-panic`);
          expect(panicResp.status).toBe(500);
          expect(await panicResp.text()).toContain("PanicError:");
        }

        // Counter should continue (not reset)
        const resp2 = await mf.dispatchFetch(`${mfUrl}durable/${uniqueId}`);
        const match2 = (await resp2.text()).match(/unstored_count: (\d+)/);
        const count2 = match2 ? parseInt(match2[1]) : 0;
        expect(count2).toBe(count1 + 1);
      }

      // Test 3: Concurrent requests with one panicking
      {
        const requests = [
          mf.dispatchFetch(`${mfUrl}test-panic`),
          mf.dispatchFetch(`${mfUrl}durable/hello`),
          mf.dispatchFetch(`${mfUrl}durable/hello`),
        ];

        const responses = await Promise.all(requests);
        
        // First should be 500 (panic), others should succeed
        expect(responses[0].status).toBe(500);
        expect(await responses[0].text()).toContain("PanicError:");
        expect(responses[1].status).toBe(200);
        expect(responses[2].status).toBe(200);
      }

    } else {
      // ===== PANIC=ABORT MODE TESTS (default) =====
      // In this mode, panics cause "Workers runtime canceled" and WASM reinitializes.

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
    }
  }, 20_000);
});
