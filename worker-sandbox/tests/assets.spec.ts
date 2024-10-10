import { describe, test, expect } from "vitest";
import { mf } from "./mf";

describe("assets", () => {
  test("assets example", async () => {
    const resp = await mf.dispatchFetch("https://fake.host/test.txt");
    const body = await resp.text();
    expect(body).toBe("TEST");
  });
})