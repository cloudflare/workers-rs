import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

describe("loopback bindings", () => {
  test("fetches through a named entrypoint binding", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}exports/get`);
    expect(resp.status).toBe(200);
    expect(await resp.json()).toEqual({
      entrypoint: "Loopback",
      greeting: "",
      env: "some value",
    });
  });

  test("passes independent props through loopback factories", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}exports/props`);
    expect(resp.status).toBe(200);
    expect(await resp.json()).toEqual(
      ["first", "second", "first"].map((greeting) => ({
        entrypoint: "Loopback",
        greeting,
        env: "some value",
      })),
    );
  });
});
