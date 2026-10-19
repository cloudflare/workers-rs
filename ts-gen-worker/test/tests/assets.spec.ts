import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

describe("assets", () => {
  test("assets example", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}asset/test.txt`);
    const body = await resp.text();
    expect(body).toBe("TEST");
  });
});
