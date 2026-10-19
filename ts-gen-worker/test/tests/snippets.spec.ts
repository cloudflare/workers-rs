import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

describe("JS Snippets", () => {
  test("performance.now", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}js_snippets/now`);
    const text = await resp.text();
    expect(text).toMatch(/^now: \d+$$/);
  });

  test("console.log", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}js_snippets/log`);
    const text = await resp.text();
    expect(text).toBe("OK");
  });
});
