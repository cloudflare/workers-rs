import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

describe("durable object websocket auto-response", () => {
  test("set and get auto-response pair", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}durable/auto-response`);
    const text = await resp.text();
    expect(text).toBe("ping:pong");
  });

  test("set and get binary auto-response pair", async () => {
    const resp = await mf.dispatchFetch(
      `${mfUrl}durable/auto-response-binary`
    );
    const text = await resp.text();
    expect(text).toBe("[1, 2, 3]:[4, 5, 6]");
  });
});
