import { afterAll, describe, expect, test } from "vitest";
import { Miniflare, Request, Response } from "miniflare";
import { manifest } from "./manifest";

function outbound(request: Request): Response {
  const url = new URL(request.url);
  if (url.host !== "upstream.test") {
    return new Response("unexpected upstream", { status: 500 });
  }
  if (url.pathname === "/data") {
    return new Response("upstream body", { status: 201 });
  }
  return new Response(`${request.method} ${request.headers.get("x-forwarded")}`);
}

const mf = new Miniflare({
  workers: [
    {
      config: {
        name: "ts-gen-worker-test",
        compatibilityDate: "2026-03-17",
        manifest,
        env: {
          GREETING: { type: "text", value: "hello from env" },
        },
      },
      dev: {
        outboundService: { type: "fetcher", handler: async (req: Request) => outbound(req) },
      },
    },
  ],
});

const get = (path: string, init?: RequestInit) =>
  mf.dispatchFetch(`http://worker.test${path}`, init);

afterAll(() => mf.dispose());

describe("ts-gen-worker", () => {
  test("hello world with ResponseInit headers", async () => {
    const res = await get("/");
    expect(res.status).toBe(200);
    expect(res.headers.get("content-type")).toBe("text/plain");
    expect(await res.text()).toBe("Hello from Rust!");
  });

  test("env binding via generated Env", async () => {
    expect(await (await get("/env")).text()).toBe("hello from env");
  });

  test("Headers append/set/delete/get/has and incoming headers", async () => {
    const res = await get("/headers", { headers: { "x-incoming": "in" } });
    expect(await res.text()).toBe(
      "multi=a, b single=1 has_deleted=false has_single=true incoming=in",
    );
    expect(res.headers.get("x-multi")).toBe("a, b");
    expect(res.headers.get("x-single")).toBe("1");
    expect(res.headers.has("x-deleted")).toBe(false);
  });

  test("request.text() and request header", async () => {
    const res = await get("/echo", {
      method: "POST",
      headers: { "x-name": "guy" },
      body: "hello",
    });
    expect(await res.text()).toBe("guy: hello");
  });

  test("request.arrayBuffer() and ArrayBuffer response", async () => {
    const bytes = new Uint8Array([1, 2, 3, 250]);
    const res = await get("/bytes", { method: "POST", body: bytes });
    expect(res.headers.get("x-sum")).toBe("256");
    expect(new Uint8Array(await res.arrayBuffer())).toEqual(bytes);
  });

  test("request.clone()", async () => {
    const res = await get("/clone", { method: "POST", body: "twice" });
    expect(await res.text()).toBe("twice|twice");
  });

  test("Response.json()", async () => {
    const res = await get("/json");
    expect(res.headers.get("content-type")).toContain("application/json");
    expect(await res.json()).toEqual({ method: "GET", n: 42 });
  });

  test("Response.redirect()", async () => {
    const res = await get("/redirect", { redirect: "manual" });
    expect(res.status).toBe(302);
    expect(res.headers.get("location")).toBe("https://example.com/elsewhere");
  });

  test("outbound fetch(url) with status/ok/text", async () => {
    expect(await (await get("/fetch-upstream")).text()).toBe("201 true upstream body");
  });

  test("outbound fetch(url, init) with method/headers/body", async () => {
    const res = await get("/fetch-init", { method: "POST", body: "payload" });
    expect(await res.text()).toBe("PUT yes");
  });

  test("ExecutionContext.waitUntil / passThroughOnException", async () => {
    expect(await (await get("/wait-until")).text()).toBe("ok");
  });

  test("console namespace", async () => {
    expect(await (await get("/log")).text()).toBe("logged");
  });

  test("request.body is null without a body", async () => {
    expect(await (await get("/stream")).text()).toBe("body=false");
  });

  test("@throws {TypeError} is matched as JsException::TypeError", async () => {
    const res = await get("/bad-header");
    expect(await res.text()).toBe("TypeError: Invalid header name.");
  });

  test("@throws {RangeError} is matched as JsException::RangeError", async () => {
    expect(await (await get("/bad-redirect")).text()).toBe("RangeError");
  });

  test("Error::Js(JsException::TypeError) via From, Display is the message", async () => {
    const body = await (await get("/error-display")).text();
    expect(body).toBe("true TypeError: Invalid header name.");
  });

  test("MessageEvent data outside string | ArrayBuffer | Blob", async () => {
    expect(await (await get("/message-event-data")).text()).toBe("other");
  });

  test("Err(Error::Rust) becomes a 500", async () => {
    const res = await get("/error");
    expect(res.status).toBe(500);
  });

  test("fallthrough 404 with status", async () => {
    const res = await get("/nope");
    expect(res.status).toBe(404);
  });
});
