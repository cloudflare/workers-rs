import { afterAll, describe, expect, test } from "vitest";
import { Miniflare, Request, Response } from "miniflare";
import { manifest } from "./manifest";

// The `#[event]` macros and the `impl` layer: routes under
// `/ext/*`, plus the scheduled and email handlers delivered via
// `unsafeTriggerHandlers` and a `#[durable_object]` class.

function outbound(request: Request): Response {
  const url = new URL(request.url);
  if (url.host !== "upstream.test") {
    return new Response("unexpected upstream", { status: 500 });
  }
  return new Response("upstream body", { status: 201 });
}

const mf = new Miniflare({
  unsafeTriggerHandlers: true,
  workers: [
    {
      config: {
        name: "ts-gen-worker-ext",
        compatibilityDate: "2026-03-17",
        manifest,
        env: {
          GREETING: { type: "text", value: "hello from env" },
          MACRO_COUNTER: {
            type: "durable-object",
            worker: "ts-gen-worker-ext",
            exportName: "MacroCounter",
          },
        },
        exports: {
          MacroCounter: { type: "durable-object", storage: "legacy-kv" },
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

describe("ts-gen-worker ext", () => {
  test("#[event(start)] ran beside the suite's own start hook", async () => {
    expect(await (await get("/ext/start")).text()).toBe("started=true");
  });

  test("#[event(scheduled)]", async () => {
    const trigger = await get("/cdn-cgi/local/scheduled?cron=*+*+*+*+*&time=1000");
    expect(trigger.status).toBe(200);
    expect(await (await get("/ext/scheduled")).text()).toBe("* * * * * 1000");
  });

  test("#[event(email)]", async () => {
    const raw = [
      "From: sender@example.com",
      "To: recipient@example.com",
      "Subject: hello",
      "Message-ID: <1@example.com>",
      "",
      "body",
    ].join("\r\n");
    const trigger = await get(
      "/cdn-cgi/local/email?from=sender@example.com&to=recipient@example.com",
      { method: "POST", body: raw },
    );
    expect(trigger.status).toBe(200);
    expect(await (await get("/ext/email")).text()).toBe(
      `sender@example.com -> recipient@example.com (${raw.length}): hello`,
    );
  });

  test("ExecutionContext::spawn keeps the invocation alive", async () => {
    expect(await (await get("/ext/spawn")).text()).toBe("spawned");
    let spawned = "";
    for (let i = 0; i < 50 && spawned === ""; i++) {
      await new Promise((r) => setTimeout(r, 50));
      spawned = await (await get("/ext/spawned")).text();
    }
    expect(spawned).toBe("done");
  });

  test("Method parse/Display/all", async () => {
    expect(await (await get("/ext/method")).text()).toBe(
      "GET Get | HEAD GET POST PUT PATCH DELETE OPTIONS CONNECT TRACE REPORT | unknown HTTP method: PURGE",
    );
  });

  test("Request::path and Request::query", async () => {
    expect(await (await get("/ext/path?name=guy&n=3")).text()).toBe("/ext/path guy 3");
    const res = await get("/ext/path?name=guy");
    expect(res.status).toBe(500);
    expect(await res.text()).toBe("missing field `n`");
  });

  test("Headers iteration, FromIterator, keys/values/getAll", async () => {
    expect(await (await get("/ext/headers-iter")).text()).toBe(
      "set-cookie=a=1,set-cookie=b=2,x-a=1, 3,x-b=2 / set-cookie,set-cookie,x-a,x-b / a=1,b=2,1, 3,2 / a=1|b=2 / getAll() can only be used with the header name \"Set-Cookie\".",
    );
  });

  test("Request::json_into and builder from_json", async () => {
    const res = await get("/ext/json-into", {
      method: "POST",
      body: JSON.stringify({ id: "x", count: 41 }),
    });
    expect(res.status).toBe(201);
    expect(res.headers.get("content-type")).toContain("application/json");
    expect(await res.json()).toEqual({ id: "x", count: 42 });
  });

  test("Request::bytes_stream and builder from_stream", async () => {
    const body = new Uint8Array(70_000).map((_, i) => i % 251);
    const res = await get("/ext/bytes-stream", { method: "POST", body });
    expect(res.headers.get("x-total")).toBe("70000");
    expect(new Uint8Array(await res.arrayBuffer())).toEqual(body);
  });

  test("Request::bytes and builder from_bytes", async () => {
    const res = await get("/ext/bytes", { method: "GET" });
    expect(new Uint8Array(await res.arrayBuffer())).toEqual(new Uint8Array(0));
  });

  test("builder from_html/empty/error/ok", async () => {
    const html = await get("/ext/html");
    expect(html.status).toBe(201);
    expect(html.headers.get("content-type")).toBe("text/html; charset=utf-8");
    expect(await html.text()).toBe("<b>hi</b>");

    const empty = await get("/ext/empty");
    expect(empty.status).toBe(204);
    expect(await empty.text()).toBe("");

    const teapot = await get("/ext/teapot");
    expect(teapot.status).toBe(418);
    expect(await teapot.text()).toBe("short and stout");

    const bad = await get("/ext/bad-error");
    expect(bad.status).toBe(500);
    expect(await bad.text()).toBe("error status codes must be in the 400-599 range, got 200");

    const ok = await get("/ext/ok-ct");
    expect(ok.headers.get("content-type")).toBe("text/csv");
    expect(await ok.text()).toBe("a,b");
  });

  test("#[durable_object] class with storage and env", async () => {
    expect(await (await get("/ext/counter")).text()).toBe("main 1 /hit hello from env");
    expect(await (await get("/ext/counter")).text()).toBe("main 2 /hit hello from env");
  });

  test("Router sync/async/params/data/env/405/404", async () => {
    expect(await (await get("/ext/router/sync/5")).text()).toBe("sync 5 7");
    const async = await get("/ext/router/async/9", { method: "POST", body: "b" });
    expect(await async.text()).toBe("async 9 b hello from env");
    expect(await (await get("/ext/router/any", { method: "DELETE" })).text()).toBe("any DELETE");
    const notAllowed = await get("/ext/router/sync/5", { method: "POST" });
    expect(notAllowed.status).toBe(405);
    const missing = await get("/ext/router/missing");
    expect(missing.status).toBe(404);
    expect(await missing.text()).toBe("Not Found");
  });

  test("#[event(fetch)] error policy: respond_with_errors shows the message", async () => {
    const rust = await get("/ext/error-rust");
    expect(rust.status).toBe(500);
    expect(await rust.text()).toBe("boom");
    const number = await get("/ext/error-number");
    expect(number.status).toBe(500);
    expect(await number.text()).toBe("42");
    const nul = await get("/ext/error-null");
    expect(nul.status).toBe(500);
    expect(await nul.text()).toBe("null");
  });

  test("#[event(fetch)] error policy: default hides the message", async () => {
    const res = await get("/ext/error-default");
    expect(res.status).toBe(500);
    expect(await res.text()).toBe("INTERNAL SERVER ERROR");
  });
});

// The Rust-native surface ported from `worker`.
// Its own instance captures workerd's structured logs for the console macros.
const logs: { level: string; message: string }[] = [];
const mf2 = new Miniflare({
  handleStructuredLogs: (log) => {
    logs.push({ level: log.level, message: log.message });
  },
  workers: [
    {
      config: {
        name: "ts-gen-worker-ext2",
        compatibilityDate: "2026-03-17",
        manifest,
        env: {
          GREETING: { type: "text", value: "hello from env" },
          EXT_JSON: { type: "json", value: { id: "j", count: 7 } },
          MACRO_COUNTER: {
            type: "durable-object",
            worker: "ts-gen-worker-ext2",
            exportName: "MacroCounter",
          },
          EXT_KV: { type: "kv" },
        },
        exports: {
          MacroCounter: { type: "durable-object", storage: "legacy-kv" },
        },
      },
      dev: {
        outboundService: { type: "fetcher", handler: async (req: Request) => outbound(req) },
      },
    },
  ],
});

const get2 = (path: string, init?: RequestInit) =>
  mf2.dispatchFetch(`http://worker.test${path}`, init);

afterAll(() => mf2.dispose());

describe("ts-gen-worker ext: Rust-native surface", () => {
  test("console_log!/debug!/warn!/error!", async () => {
    expect(await (await get2("/ext/console")).text()).toBe("logged");
    for (let i = 0; i < 50 && logs.length < 4; i++) {
      await new Promise((r) => setTimeout(r, 20));
    }
    const ext = logs.filter((l) => l.message.startsWith("ext "));
    expect(ext).toEqual([
      { level: "log", message: "ext log 1 a" },
      { level: "debug", message: "ext debug 2" },
      { level: "warn", message: "ext warn 3" },
      { level: "error", message: "ext error 4" },
    ]);
  });

  test("Delay resolves after its duration; a dropped Delay is cancelled", async () => {
    expect(await (await get2("/ext/delay")).text()).toBe("true true");
  });

  test("DateInit and DateExt on js_sys::Date", async () => {
    expect(await (await get2("/ext/date")).text()).toBe(
      "1630611511000 1630611511000 2021-09-02T19:38:31.000Z true",
    );
  });

  test("Cors applies to a Response", async () => {
    const res = await get2("/ext/cors");
    expect(await res.text()).toBe("untouched=0");
    expect(res.headers.get("access-control-allow-credentials")).toBe("true");
    expect(res.headers.get("access-control-max-age")).toBe("600");
    expect(res.headers.get("access-control-allow-origin")).toBe("https://a.test,https://b.test");
    expect(res.headers.get("access-control-allow-methods")).toBe("GET,POST");
    expect(res.headers.get("access-control-allow-headers")).toBe("x-a,x-b");
    expect(res.headers.get("access-control-expose-headers")).toBe("x-c");
  });

  test("EnvExt dynamic lookup: var/secret/get, missing and mistyped", async () => {
    expect(await (await get2("/ext/env-ext")).text()).toBe(
      "hello from env | hello from env | x | Binding `NOPE` is undefined. | Binding `MACRO_COUNTER` is not a string." +
        ' | Body { id: "j", count: 7 }',
    );
  });

  test("#[send], SendWrapper, IntoSendFuture", async () => {
    expect(await (await get2("/ext/send")).text()).toBe("send 1 /send hello from env | w | 1");
  });

  test("Fetch::Url/Request send and send_with_signal; AbortController::default", async () => {
    expect(await (await get2("/ext/fetch-enum")).text()).toBe(
      "201 upstream body 201 upstream body | AbortError: The operation was aborted",
    );
  });

  test("KVNamespace fluent builders: put/get/get_bulk/list", async () => {
    expect(await (await get2("/ext/kv")).text()).toBe(
      'Some("text") Some("text") Some(Meta { tag: "m" }) None Some(42) Some(Body { id: "k", count: 3 }) Some([1, 2, 3]) None true false' +
        ' | [("missing", None), ("n", Some("42")), ("t", Some("text"))]' +
        ' | [("n", (Some("42"), None)), ("t", (Some("text"), Some(Meta { tag: "m" })))] | ["t"] true 4 3',
    );
  });

  test("ok::Ok", async () => {
    const res = await get2("/ext/ok");
    expect(res.status).toBe(202);
    expect(await res.text()).toBe("ok");
  });

  test("Response::into_parts and builder.body re-construct with changes", async () => {
    expect(await (await get2("/ext/into-parts")).text()).toBe(
      '404 yes 1 upstream body {"foo":"bar"} | 204 true',
    );
  });

  test("builder header/cors/encode_body chain", async () => {
    expect(await (await get2("/ext/builder-header")).text()).toBe("2 text/csv https://a.test");
  });

  test("Request::new_with_method", async () => {
    expect(await (await get2("/ext/request-method")).text()).toBe("PUT https://upstream.test/x");
  });

  test("builder.from_websocket", async () => {
    expect(await (await get2("/ext/from-websocket")).text()).toBe("101 true");
  });

  test("DurableObjectStorage::get_into / put_serialized", async () => {
    expect(await (await get2("/ext/storage-serde")).text()).toBe(
      'Some(Body { id: "s", count: 1 }) Some(7) None',
    );
    expect(await (await get2("/ext/storage-serde")).text()).toBe(
      'Some(Body { id: "s", count: 2 }) Some(7) None',
    );
  });

  test("http round trip hands the original ReadableStream back", async () => {
    const res = await get2("/ext/http-roundtrip", { method: "POST", body: "payload" });
    expect(await res.text()).toBe("true payload | true stream | one true two");
  });

  test("Headers <-> HeaderMap is fallible on a non-ASCII value", async () => {
    expect(await (await get2("/ext/header-map")).text()).toBe(
      "1, 2 1 | header `x-bin` has a non-ASCII value: failed to convert header to a str" +
        " | header `x-del` has an invalid value: failed to parse header value",
    );
  });
});
