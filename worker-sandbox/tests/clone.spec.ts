import { describe, test, expect } from "vitest";
import { mf } from "./mf";

describe("cache", () => {
  test("cloned", async () => {
    const resp = await mf.dispatchFetch("https://fake.host/cloned");
    expect(await resp.text()).toBe("true");
  });

  test("cloned stream", async () => {
    const resp = await mf.dispatchFetch("https://fake.host/cloned-stream");
    expect(await resp.text()).toBe("true");
  });

  test("cloned fetch", async () => {
    const resp = await mf.dispatchFetch("https://fake.host/cloned-fetch");
    expect(await resp.text()).toBe("true");
  });
});
