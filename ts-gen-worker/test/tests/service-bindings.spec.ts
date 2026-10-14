import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

describe("service bindings", () => {
  test("by path", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}remote-by-path`);
    expect(await resp.text()).toBe("hello world");
  });

  test("by request", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}remote-by-request`);
    expect(await resp.text()).toBe("hello world");
  });
});
