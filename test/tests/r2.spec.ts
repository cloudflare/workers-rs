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

  test("list without include", async () => {
    // Without `include`, a listing does not return the objects' metadata (workerd's
    // `r2_list_honor_include` flag, 2022-08-04): workerd omits `httpMetadata` / `customMetadata`
    // when the R2 backend omits them, while Miniflare's backend still sends them as empty objects.
    // Both shapes must read as empty metadata on the Rust side.
    const bucket = await mf.getR2Bucket("SEEDED_BUCKET");
    await bucket.put("js-with-props", "example", {
      httpMetadata: { contentType: "text/text" },
      customMetadata: { a: "b" },
    });
    const listed = (await bucket.list()).objects.find((o) => o.key === "js-with-props")!;
    expect(listed.httpMetadata?.contentType).toBeUndefined();
    expect(listed.customMetadata?.a).toBeUndefined();
    await bucket.delete("js-with-props");

    const resp = await mf.dispatchFetch(`${mfUrl}r2/list-without-include`);
    expect(await resp.text()).toBe("content_type=None custom={}");
  });

  test("get empty", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}r2/get-empty`);
    expect(await resp.text()).toBe("ok");
  });

  test("get", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}r2/get`);
    expect(await resp.text()).toBe("ok");
  });

  test("get with offset and length range", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}r2/get-range/offset-length`);
    expect(await resp.text()).toBe("Ok(OffsetWithLength { offset: 2, length: 3 })");
  });

  test("get with suffix range", async () => {
    // The runtime echoes a suffix read as the resolved `{offset, length}`.
    const resp = await mf.dispatchFetch(`${mfUrl}r2/get-range/suffix`);
    expect(await resp.text()).toBe("Ok(OffsetWithLength { offset: 7, length: 3 })");
  });

  test("get without range", async () => {
    // Miniflare echoes the whole object as the range when none was requested;
    // workerd omits `range` there and `Object::range()` returns `Err`.
    const resp = await mf.dispatchFetch(`${mfUrl}r2/get-range/none`);
    expect(await resp.text()).toBe("Ok(OffsetWithLength { offset: 0, length: 10 })");
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
