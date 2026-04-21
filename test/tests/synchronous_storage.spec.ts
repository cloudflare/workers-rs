import {describe, test, expect} from "vitest";
import { mf, mfUrl } from "./mf";

describe("synchronous api durable object", () => {
  test("synchronous-storage", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}synchronous-storage`);
    expect(resp.status).toBe(204);
  });
});
