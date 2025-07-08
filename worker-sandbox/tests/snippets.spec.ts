import { describe, test, expect } from "vitest";
import { mf } from "./mf";

describe("JS Snippets", () => {
  test("performance.now", async () => {
    const resp = await mf.dispatchFetch("http://fake.host/js_snippets/now");
    const text = await resp.text();
    expect(text).toMatch(/^now: \d+$$/);
  });

  test("console.log", async () => {
    const resp = await mf.dispatchFetch("http://fake.host/js_snippets/log");
    const text = await resp.text();
    expect(text).toBe("OK");
  });
});
