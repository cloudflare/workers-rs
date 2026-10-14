import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

describe("analytics engine", () => {
  test("write data point", async () => {
    const resp = await mf.dispatchFetch(
      `${mfUrl}analytics-engine`,
    );
    console.log(await resp.text())
    expect(resp.status).toBe(200);
  });
});
