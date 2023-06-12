import { describe, test, expect } from "vitest";
import { mf } from "./mf";

describe("subrequest", () => {
  test("request init fetch", async () => {
    const resp = await mf.dispatchFetch("https://fake.host/request-init-fetch");
    expect(resp.status).toBe(200);
  });

  test("cancelled fetch", async () => {
    const resp = await mf.dispatchFetch("https://fake.host/cancelled-fetch");
    expect(await resp.text()).toContain("AbortError");
  });

  test("fetch timeout", async () => {
    const resp = await mf.dispatchFetch("https://fake.host/fetch-timeout");
    expect(await resp.text()).toBe("Cancelled");
  });

  test.skip("request init fetch post", async () => {
    const resp = await mf.dispatchFetch(
      "https://fake.host/request-init-fetch-post"
    );
    expect(await resp.json()).toMatchObject({
      url: "https://httpbin.org/post",
    });
  });
});
