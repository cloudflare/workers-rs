import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

describe("secret store", () => {
  test("secret store", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}get-from-secret-store`);
    expect(await resp.text()).toBe("secret value");
  });

  test("get_from_secret_store_missing", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}get-from-secret-store-missing`);
    expect(resp.status).toBe(500);
  });
});
