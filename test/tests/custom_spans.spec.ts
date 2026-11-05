import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

// Tracing is not enabled on the test worker, so every span is a no-op as far
// as recording goes. What these tests pin down is the binding contract: the
// runtime API is reachable from Rust, callbacks run exactly once and return
// their values, handles stay usable after `end()`, and nothing throws.
describe("custom spans", () => {
  test("enter_span runs the body once and returns its value", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}tracing/enter-span`);
    expect(resp.status).toBe(200);
    expect(await resp.json()).toEqual({ value: 42, calls: 1 });
  });

  test("enter_span_async drives the future across an await", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}tracing/enter-span-async`);
    expect(resp.status).toBe(200);
    expect(await resp.json()).toEqual({ value: "done" });
  });

  test("start_span is ended explicitly and idempotently", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}tracing/start-span`);
    expect(resp.status).toBe(200);
    expect(await resp.json()).toEqual({
      traced_before_end: false,
      traced_after_end: false,
    });
  });

  test("start_active_span hands out a handle that outlives the body", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}tracing/start-active-span`);
    expect(resp.status).toBe(200);
    expect(await resp.json()).toEqual({ child_value: 5 });
  });

  test("active_span resolves inside and outside custom spans", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}tracing/active-span`);
    expect(resp.status).toBe(200);
    expect(await resp.json()).toEqual({
      root_present: true,
      inside_present: true,
    });
  });
});
