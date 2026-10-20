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
