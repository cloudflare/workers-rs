import { describe, expect, test } from "vitest";
import { mf, mfUrl } from "./mf";

describe("serde_repro", () => {
  test("serde_json::Value serializes to Object with json_compatible", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}serde-repro`);
    expect(resp.status).toBe(200);
    const body = (await resp.json()) as {
      default: string;
      json_compatible: string;
    };
    expect(body.default).toBe("Map");
    expect(body.json_compatible).toBe("Object");
  });
});
