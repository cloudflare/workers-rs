import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

describe("r2", () => {
  test("list empty", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}r2/list-empty`);
    expect(await resp.text()).toBe("ok");
  });

  test("list", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}r2/list`);
    expect(await resp.text()).toBe("ok");
  });

  test("get empty", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}r2/get-empty`);
    expect(await resp.text()).toBe("ok");
  });

  test("get", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}r2/get`);
    expect(await resp.text()).toBe("ok");
  });

  test("get many sequential", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}r2/get-many-sequential`);
    expect(resp.status).toBe(200);

    const body = (await resp.json()) as {
      mode: string;
      count: number;
      elapsed_ms: number;
    };

    expect(body.mode).toBe("sequential");
    expect(body.count).toBe(512);
    expect(body.elapsed_ms).toBeGreaterThanOrEqual(0);
  });

  test("get many parallel", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}r2/get-many-parallel`);
    expect(resp.status).toBe(200);

    const body = (await resp.json()) as {
      mode: string;
      count: number;
      elapsed_ms: number;
    };

    expect(body.mode).toBe("parallel");
    expect(body.count).toBe(512);
    expect(body.elapsed_ms).toBeGreaterThanOrEqual(0);
  });

  test("get many chunked", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}r2/get-many-chunked`);
    expect(resp.status).toBe(200);

    const body = (await resp.json()) as {
      mode: string;
      count: number;
      elapsed_ms: number;
    };

    expect(body.mode).toBe("chunked");
    expect(body.count).toBe(512);
    expect(body.elapsed_ms).toBeGreaterThanOrEqual(0);
  });

  test("put", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}r2/put`, {
      method: "put",
    });
    expect(await resp.text()).toBe("ok");
  });

  test("put properties", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}r2/put-properties`, {
      method: "put",
    });
    expect(await resp.text()).toBe("ok");
  });

  test("delete", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}r2/delete`, {
      method: "delete",
    });
    expect(await resp.text()).toBe("ok");
  });
});
