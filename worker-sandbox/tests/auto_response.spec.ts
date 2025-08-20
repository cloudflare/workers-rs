import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

describe("durable object websocket auto-response", () => {
  test("set and get auto-response pair", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}durable/auto-response`);
    const text = await resp.text();
    expect(text).toBe("ping:pong");
  });
});
