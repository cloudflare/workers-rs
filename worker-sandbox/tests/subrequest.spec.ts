import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

describe("subrequest", () => {
  test("request init fetch", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}request-init-fetch`);
    expect(resp.status).toBe(200);
  });

  test("cancelled fetch", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}cancelled-fetch`);
    expect(await resp.text()).toContain("AbortError");
  });

  test("fetch timeout", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}fetch-timeout`);
    expect(await resp.text()).toBe("Cancelled");
  });

  test.skip("request init fetch post", async () => {
    const resp = await mf.dispatchFetch(
      `${mfUrl}request-init-fetch-post`
    );
    expect(await resp.json()).toMatchObject({
      url: "https://httpbin.org/post",
    });
  });
});
