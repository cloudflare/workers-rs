import { describe, test, expect } from "vitest";
import { mf } from "./mf";

describe("clone", () => {
  test("clone", async () => {
    const resp = await mf.dispatchFetch("https://fake.host/clone", {
      method: "POST",
      body: "testing"
    });
    expect(await resp.text()).toBe("testing");
  });

  test("clone inner", async () => {
    const resp = await mf.dispatchFetch("https://fake.host/clone-inner", {
      method: "POST",
      body: "testing"
    });
    expect(await resp.text()).toBe("testing");
  });
});
