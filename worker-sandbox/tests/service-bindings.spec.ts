import { describe, test, expect } from "vitest";
import { mf } from "./mf";

describe("service bindings", () => {
  test("by path", async () => {
    const resp = await mf.dispatchFetch("https://fake.host/remote-by-path");
    expect(await resp.text()).toBe("hello world");
  });

  test("by request", async () => {
    const resp = await mf.dispatchFetch("https://fake.host/remote-by-request");
    expect(await resp.text()).toBe("hello world");
  });
});
