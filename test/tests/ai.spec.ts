import { describe, expect, test } from "vitest";
import { mf, mfUrl } from "./mf";

describe("ai", () => {
  test("run sends maps as JSON objects", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}ai/run-input`);
    expect(resp.status).toBe(200);
    expect(JSON.parse(await resp.text())).toEqual({
      prompt: "hi",
      max_tokens: null,
      tool: { parameters: { type: "object", properties: {} } },
    });
  });
});
