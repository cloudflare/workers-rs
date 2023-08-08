import { describe, test, expect } from "vitest";
import { mf } from "./mf";

describe("durable", () => {
  test("put-raw", async () => {
    const resp = await mf.dispatchFetch("https://fake.host/durable/put-raw");
    expect(await resp.text()).toBe("ok");
  });
});
