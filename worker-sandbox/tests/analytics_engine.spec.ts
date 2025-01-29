import { describe, test, expect } from "vitest";
import { mf } from "./mf";

describe("analytics engine", () => {
  test("write data point", async () => {
    const resp = await mf.dispatchFetch(
      `https://fake.host/analytics-engine`,
    );
    console.log(await resp.text())
    expect(resp.status).toBe(200);
  });
});
