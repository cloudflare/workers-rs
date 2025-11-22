import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

describe("rate limit", () => {
  test("basic rate limit check", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}rate-limit/check`);
    expect(resp.status).toBe(200);
    const data = await resp.json() as { success: boolean };
    expect(data).toHaveProperty("success");
    expect(data.success).toBe(true);
  });

  test("rate limit with custom key", async () => {
    const key = "test-key-123";
    const resp = await mf.dispatchFetch(`${mfUrl}rate-limit/key/${key}`);
    expect(resp.status).toBe(200);
    const data = await resp.json() as { success: boolean; key: string };
    expect(data).toHaveProperty("success");
    expect(data).toHaveProperty("key");
    expect(data.key).toBe(key);
    expect(data.success).toBe(true);
  });

  test("different keys have independent limits", async () => {
    // Test that different keys have separate rate limits
    const key1 = "user-1";
    const key2 = "user-2";

    const resp1 = await mf.dispatchFetch(`${mfUrl}rate-limit/key/${key1}`);
    const resp2 = await mf.dispatchFetch(`${mfUrl}rate-limit/key/${key2}`);

    expect(resp1.status).toBe(200);
    expect(resp2.status).toBe(200);

    const data1 = await resp1.json() as { success: boolean; key: string };
    const data2 = await resp2.json() as { success: boolean; key: string };

    expect(data1.success).toBe(true);
    expect(data2.success).toBe(true);
    expect(data1.key).toBe(key1);
    expect(data2.key).toBe(key2);
  });

  test("bulk rate limit test", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}rate-limit/bulk-test`);
    expect(resp.status).toBe(200);
    const data = await resp.json() as { results: Array<{ index: number; key: string; success: boolean }> };
    expect(data).toHaveProperty("results");
    expect(Array.isArray(data.results)).toBe(true);
    expect(data.results.length).toBe(15);

    // Check that results have the expected structure
    data.results.forEach((result, index: number) => {
      expect(result).toHaveProperty("index");
      expect(result).toHaveProperty("key");
      expect(result).toHaveProperty("success");
      expect(result.index).toBe(index);
      expect(typeof result.success).toBe("boolean");
    });

    // We're using 3 different keys (bulk-test-0, bulk-test-1, bulk-test-2)
    // with a limit of 10 per 60 seconds. Each key is used 5 times (15 requests / 3 keys).
    // All requests should succeed since each key stays under the limit of 10.

    // Group results by key
    const resultsByKey: Record<string, Array<{ index: number; key: string; success: boolean }>> = {};
    data.results.forEach((result) => {
      if (!resultsByKey[result.key]) {
        resultsByKey[result.key] = [];
      }
      resultsByKey[result.key].push(result);
    });

    // Should have exactly 3 keys
    expect(Object.keys(resultsByKey).length).toBe(3);

    // Each key should have 5 requests, all successful (under limit of 10)
    Object.entries(resultsByKey).forEach(([key, results]) => {
      expect(results.length).toBe(5);
      results.forEach((result) => {
        expect(result.success).toBe(true);
      });
    });
  });

  test("rate limit reset with unique keys", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}rate-limit/reset`);
    expect(resp.status).toBe(200);
    const data = await resp.json() as Record<string, boolean>;

    // Should have 12 request results
    expect(Object.keys(data).length).toBe(12);

    // Check that we have the expected keys
    for (let i = 1; i <= 12; i++) {
      expect(data).toHaveProperty(`request_${i}`);
      expect(typeof data[`request_${i}`]).toBe("boolean");
    }

    // With a limit of 10 per 60 seconds, the first 10 requests MUST succeed
    // and requests 11 and 12 MUST fail
    for (let i = 1; i <= 10; i++) {
      expect(data[`request_${i}`]).toBe(true);
    }

    // Requests 11 and 12 must be rate limited
    expect(data["request_11"]).toBe(false);
    expect(data["request_12"]).toBe(false);
  });

  test("multiple rapid requests with same key", async () => {
    // Generate a unique key for this test
    const testKey = `rapid-test-${Date.now()}`;

    // Make multiple rapid requests with the same key
    const promises = [];
    for (let i = 0; i < 5; i++) {
      promises.push(mf.dispatchFetch(`${mfUrl}rate-limit/key/${testKey}`));
    }

    const responses = await Promise.all(promises);

    // All responses should be successful (200 status)
    responses.forEach(resp => {
      expect(resp.status).toBe(200);
    });

    // Parse the responses
    const results = await Promise.all(responses.map(r => r.json())) as Array<{ success: boolean; key: string }>;

    // All should have the same key
    results.forEach(data => {
      expect(data.key).toBe(testKey);
      expect(data).toHaveProperty("success");
    });

    // With limit of 10, all 5 requests should succeed
    results.forEach((data) => {
      expect(data.success).toBe(true);
    });
  });

  test("sequential requests enforce rate limit", async () => {
    // Generate a unique key for this test to avoid interference
    const testKey = `sequential-test-${Date.now()}`;

    // Make 15 sequential requests with the same key
    // With a limit of 10 per 60 seconds, first 10 should succeed, rest should fail
    const results: Array<{ success: boolean; key: string }> = [];
    for (let i = 0; i < 15; i++) {
      const resp = await mf.dispatchFetch(`${mfUrl}rate-limit/key/${testKey}`);
      expect(resp.status).toBe(200);
      const data = await resp.json() as { success: boolean; key: string };
      results.push(data);
    }

    // Verify first 10 requests succeed
    for (let i = 0; i < 10; i++) {
      expect(results[i].success).toBe(true);
      expect(results[i].key).toBe(testKey);
    }

    // Verify requests 11-15 are rate limited
    for (let i = 10; i < 15; i++) {
      expect(results[i].success).toBe(false);
      expect(results[i].key).toBe(testKey);
    }
  });
});
