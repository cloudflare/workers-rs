import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

describe("CfProperties", () => {
  test("fetch with cache_ttl=-1 and CacheMode::NoStore", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}fetch-cache-ttl-negative`);
    expect(resp.status).toBe(200);
    const text = await resp.text();
    expect(text).toContain("succeeded with status");
    // Verify the fetch to google.com was successful (should return 200 or similar)
    expect(text).toMatch(/status (200|301|302)/);
  });
});
