import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

describe("synchronous api durable object", () => {
  test("smoke", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}synchronous-storage/smoke`);
    expect(resp.status).toBe(200);
    expect(await resp.text()).toBe("smoke ok");
  });

  test("overwrite", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}synchronous-storage/overwrite`);
    expect(resp.status).toBe(200);
    expect(await resp.text()).toBe("overwrite ok");
  });

  test("not_found", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}synchronous-storage/not_found`);
    expect(resp.status).toBe(200);
    expect(await resp.text()).toBe("not_found ok");
  });

  test("list", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}synchronous-storage/list`);
    expect(resp.status).toBe(200);
    expect(await resp.text()).toBe("list ok");
  });

  describe.sequential("persist", () => {
    test("fill", async () => {
      const resp = await mf.dispatchFetch(
        `${mfUrl}synchronous-storage/persist_fill`
      );
      expect(resp.status).toBe(200);
      expect(await resp.text()).toBe("persist_fill ok");
    });

    test("check", async () => {
      const resp = await mf.dispatchFetch(
        `${mfUrl}synchronous-storage/persist_check`
      );
      expect(resp.status).toBe(200);
      expect(await resp.text()).toBe("persist_check ok");
    });

    test("cleanup", async () => {
      const resp = await mf.dispatchFetch(
        `${mfUrl}synchronous-storage/persist_cleanup`
      );
      expect(resp.status).toBe(200);
      expect(await resp.text()).toBe("persist_cleanup ok");
    });
  });
});