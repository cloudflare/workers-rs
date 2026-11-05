import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

// Tracing is not enabled on the test worker, so spans are not recorded. Nesting is still
// observable: the active span is compared by identity with the span that should be active.

describe("custom spans", () => {
  test("sync spans nest, return their values, and restore the enclosing span", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}tracing/spans`);
    expect(resp.status).toBe(200);
    expect(await resp.json()).toEqual({
      value: 42,
      calls: 1,
      outer_active: true,
      inner_active: true,
      manual_active_before: false,
      manual_active_in_scope: true,
      root_restored: true,
    });
  });

  test("async spans stay active across awaits", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}tracing/async-spans`);
    expect(resp.status).toBe(200);
    expect(await resp.json()).toEqual({
      before: true,
      after: true,
      nested: true,
      value: 8,
      root_restored: true,
    });
  });

  test("concurrent spans each see their own span on every poll", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}tracing/concurrent-spans`);
    expect(resp.status).toBe(200);
    expect(await resp.json()).toEqual({
      a: Array(4).fill(true),
      b: Array(4).fill(true),
    });
  });
});
