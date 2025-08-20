import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

describe("cache", () => {
  test("cache example", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}cache-example`);
    const { timestamp } = (await resp.json()) as { timestamp: unknown };

    for (let i = 0; i < 5; i++) {
      const resp = await mf.dispatchFetch(`${mfUrl}cache-example`);
      const data = (await resp.json()) as { timestamp: unknown };

      expect(data.timestamp).toBe(timestamp);
    }
  });

  test("cache stream", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}cache-stream`);
    const body = await resp.text();

    for (let i = 0; i < 5; i++) {
      const resp = await mf.dispatchFetch(`${mfUrl}cache-stream`);
      const cachedBody = await resp.text();

      expect(cachedBody).toBe(body);
    }
  });

  test("cache api", async () => {
    const key = "example.org";
    const getEndpoint = `${mfUrl}cache-api/get/${key}`;
    const putEndpoint = `${mfUrl}cache-api/put/${key}`;
    const deleteEndpoint = `${mfUrl}cache-api/delete/${key}`;

    // First time should result in cache miss
    let resp = await mf.dispatchFetch(getEndpoint);
    expect(await resp.text()).toBe("cache miss");

    // Add key to cache
    resp = await mf.dispatchFetch(putEndpoint, { method: "put" });
    const { timestamp: expectedTimestamp } = (await resp.json()) as {
      timestamp: unknown;
    };

    // Should now be cache hit
    resp = await mf.dispatchFetch(getEndpoint);
    let data = (await resp.json()) as { timestamp: unknown };
    expect(data.timestamp).toBe(expectedTimestamp);

    // Delete key from cache
    resp = await mf.dispatchFetch(deleteEndpoint, { method: "post" });
    expect(await resp.text()).toBe('"Success"');

    // Delete key from cache
    resp = await mf.dispatchFetch(getEndpoint);
    expect(await resp.text()).toBe("cache miss");

    // Another delete should fail
    resp = await mf.dispatchFetch(deleteEndpoint, { method: "post" });
    expect(await resp.text()).toBe('"ResponseNotFound"');
  });
});
