import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

describe("cache", () => {
  test("cloned", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}cloned`);
    expect(await resp.text()).toBe("true");
  });

  test("cloned stream", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}cloned-stream`);
    expect(await resp.text()).toBe("true");
  });

  test("cloned fetch", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}cloned-fetch`);
    expect(await resp.text()).toBe("true");
  });

  test("cloned response", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}cloned-response`);
    expect(await resp.text()).toBe("true");
  });
});
