# Test suite port (workers-rs-5)

Branch `gbedford/tgw-test-port`; ts-gen branch `agent/test-port`.

The `test/` suite is copied verbatim into `ts-gen-worker/test/tests/` and the Rust
worker is re-implemented in `ts-gen-worker/test/src/` against the generated API.
The port deliberately uses *only* `ts-gen-worker` plus a small test-local
`util.rs`; every helper in that file is a `worker` ergonomic that the crate lacks,
so it doubles as the concrete list for workers-rs-4.

Result: **19 of 24 spec files run green (122 tests); 5 are skipped** (`d1`,
`r2`, `send_email`, `panic`, `signal`) plus the self-skipping `container`.
Every skip carries a `// GAP:` comment (`grep -rn "GAP:" ts-gen-worker/test`).

## Done

- Harness: `tests/mf.ts` / `tests/mf-socket.ts` are the originals pointed at the
  module-target build through `tests/manifest.ts`, which mirrors the whole
  `build/` dir (wasm-bindgen `inline_js` snippets land in `build/snippets/`).
  `public/test.txt` copied. `worker.spec.ts` kept green: it had to drop
  `type: "worker"` (the pinned miniflare 5.20260923.0-alpha rejects the key; the
  spec was failing before this branch) and its `/fetch` route moved to
  `/fetch-upstream` to make room for the suite's `/fetch`.
- `types/worker.d.ts`: added, member-verbatim from `@cloudflare/workers-types`,
  everything the ported modules touch: Body members on Request/Response, BodyInit,
  AbortController/Signal, Blob/File/FormData, streams (reader/writer/Transform/
  FixedLength/Identity), `setTimeout`/`clearTimeout`, Cache API, `IncomingRequestCfProperties`
  (trimmed), `Fetcher`, KV, SecretsStoreSecret, RateLimit, AnalyticsEngine,
  Queue/Message/MessageBatch, DurableObject{Id,Namespace,Stub,State,Storage}
  + options, SqlStorage/SqlStorageCursor, Event/EventTarget/MessageEvent/CloseEvent,
  WebSocket/WebSocketPair, Socket + `cloudflare:sockets`. Chompfile now passes
  `--export ts-gen-worker/types/worker.d.ts --export cloudflare:sockets`.
- `test/types/env.d.ts`: every binding in `mf.ts`; generated through per-type
  `--external X=::ts_gen_worker::X` (see Decisions). `DB`, `*_BUCKET`, `EMAIL`
  are `any` until those areas are declared.
- ts-gen fixes on `agent/test-port` (each a self-contained commit with fixture,
  snapshot, CONVENTIONS):
  - `4985806` string enums emitted `#[wasm_bindgen(js_name)] Variant`, which
    wasm-bindgen rejects; now `Variant = "value"`, passed by value (enums are
    `Copy` and have no `&T: IntoWasmAbi`). Fluent builder methods for optional
    `ArrayBufferView` props now carry the setter's `<T: TypedArray>` bound
    (was `&Object`, never type-checked). Includes a compile-only integration test.
  - `213f56a` `export { _connect as connect }` kept `js_name = "_connect"`;
    renamed function exports now bind to the public name.
  - `9909da5` getters whose key sanitises (`0` → `_0`, `$public`, `@cf/...`)
    emitted no `js_name`; `WebSocketPair[0]` read `_0`.
- Rust port, module by module (table below), dispatching on
  `(method, path segments)` with slice patterns in `lib.rs`; DO classes are
  `#[wasm_bindgen]` structs with `constructor` + `async fn fetch(&self)` and
  `webSocket*` methods, exported by class name; `queue` and `connect` handlers
  are `js_namespace = "default"` exports like `fetch`.

## Module status

Legend: ported = all routes in the module run and their specs pass.

| module | status | `worker` API used → how it was ported / what is missing |
| --- | --- | --- |
| `request.rs` | ported | `Request::path/url/cf/bytes/text/stream`, `Response::ok/error/from_json/from_bytes/from_body/from_stream/redirect/cloned`, `ResponseBuilder::with_status/with_headers`, `Env::secret/var/object_var`, `Date`, `Delay`, `console_log!`. Streams (`/xor`, `/cloned-stream`) hand-built with `TransformStream` + `spawn_local`; `Delay` is `setTimeout` + `Promise`; `object_var` via `serde_wasm_bindgen`. |
| `fetch.rs` | ported | `Fetch::Url/Request`, `send_with_signal`, `AbortController`, `RequestInit.method`, `Response::json/cloned/cf/encode_body`. `encode_body()` is Rust-only state in `worker::Response` (no JS getter); the port checks `cf` round-trip only. |
| `js_snippets.rs` | ported | `inline_js` works under `--target module` once the harness ships `build/snippets/`. |
| `user.rs` | ported | `Date::new(DateInit::Millis/String)`, `Date::now()` → `js_sys::Date`. |
| `form.rs` | ported | `Request::form_data`, `FormEntry::{Field,File}`, `File::name/bytes`, `kv.put/get/list`. `FormData.get` returns `JsValue` (see gaps); a local `FormEntry` does the split. `env.secret(&dynamic_name)` has no generated equivalent → `Reflect::get`. |
| `cache.rs` | ported | `Cache::default/get/put/delete`, `CacheDeletionOutcome` → `caches.default` + `CacheQueryOptions{ignoreMethod}`; outcome strings reproduced by hand. |
| `signal.rs` | **skipped** | `signal::is_registered/is_near_cpu_limit` are shim state (worker-build). No binding corresponds; needs the shim/runtime story (workers-rs-4 / Guy). |
| `service.rs` | ported | `Fetcher::fetch_request/fetch` → `Fetcher::fetch` / `fetch_with_str_and_init`. |
| `kv.rs` | ported (7 spec'd routes + 3 bulk routes) | `KvStore::get().text()/text_with_metadata/json`, `put().metadata/expiration/execute`, `list().prefix`, `get_bulk`. Bulk needs the `keys: string[]` overloads; their `Map` return collapses (gaps). `get` returning `null` is **not** `None` through `JsOption` (open problem). |
| `secret_store.rs` | ported | `SecretStore::get` → `SecretsStoreSecret::get` (throws on missing; 500 either way). |
| `rate_limit.rs` | ported | `RateLimiter::limit(key)` → `RateLimit::limit(&RateLimitOptions::new(key))`. |
| `assets.rs` | ported | `env.assets().fetch(url)` → `Fetcher::fetch_with_str`. |
| `analytics_engine.rs` | ported | `AnalyticsEngineDataPointBuilder::indexes/add_blob/add_double/write_to` → dictionary builder with `&[JsValue]` blobs/indexes. |
| `queue.rs` | ported | `#[event(queue)]`, `MessageBatch<T>::messages`, `Message::body/id/timestamp/into_body`, `Queue::send/send_batch`. Typed body via `serde_wasm_bindgen`; `sendBatch(Iterable<MessageSendRequest>)` erases to `&JsValue` (pass a `js_sys::Array`). `batch send` test is timing-sensitive under full-suite load (1.2 s wait vs 1 s batch timeout), same as the original. |
| `durable.rs` | ported | `#[durable_object]`, `State::id/storage/block_concurrency_while`, `Storage::get/put/list/get_multiple/put_multiple/put_raw/put_multiple_raw/delete/delete_multiple/delete_all`, `ObjectNamespace::id_from_name/unique_id/get_by_name(_with_location_hint)`, `Stub::fetch_with_str/fetch_with_request`, `Request::new_with_init`. `block_concurrency_while` re-implemented in `util.rs` (Closure → `Function<fn() -> Promise>`); constructor-time gating works via `spawn_local` but only because the microtask runs before the first event (gap: async bindings have no sync `Promise` form). `env.durable_object("BAD")` erroring has no equivalent (Env getters are static) → `Reflect::get` is `undefined`. |
| `counter.rs` | ported | `State::accept_web_socket`, `WebSocket::serialize_attachment/deserialize_attachment/send_with_str/close/events`, `websocket_message/close/error` DO methods, `Response::from_websocket`, `ResponseBuilder::with_websocket`, `Request::headers_mut`. Event streams via `addEventListener` closures (`util::on_websocket_events`). |
| `alarm.rs` | ported (no spec) | `Storage::set_alarm(Duration)` → `set_alarm(f64 ms)`. `ALARM` isn't bound in `mf.ts`, as in the original. |
| `put_raw.rs` | ported | `Storage::put_raw/put_multiple_raw` → `put(&JsValue)` / `put_with_entries(&Object)`. |
| `auto_response.rs` | ported | `State::set_websocket_auto_response/get_websocket_auto_response`, `WebSocketRequestResponsePair`. |
| `sql_counter.rs`, `sql_iterator.rs` | ported | `SqlStorage::exec(query, Option<Vec<SqlStorageValue>>)`, `SqlCursor::to_array/next/raw/column_names`, `SqlStorageValue` (+ `try_from_i64`, `Debug` format the specs assert on). Re-implemented in test-local `sql.rs` over `exec(&[JsValue])`, `to_array() -> Vec<JsValue>`, `raw() -> Iterator<Array>`. |
| `ws.rs` | ported | `WebSocketPair`, `WebSocket::accept/events/send_with_str`, `WebsocketEvent::{Message,Close}`, `WebSocket::connect` (upgrade fetch). |
| `socket.rs` | ported (4/5 socket tests; 1 pre-existing skip) | `#[event(connect)]` / `connect(udp)`, `Socket: AsyncRead + AsyncWrite`, `UdpSocket::recv/send`, `ConnectionBuilder::connect`, `Socket::opened/closed/close`. Byte I/O done directly on reader/writer; `read_exact`/`write_all` live in the module. One `connect` export dispatches on `socket.protocol` like worker-build's shim. |
| `d1.rs` | **skipped** | `D1Database::prepare/batch/exec/dump/with_session(_constraint)`, `D1PreparedStatement::bind_refs/all/first/raw/run`, `query!`, `D1Type`, `D1PreparedArgument`, `D1Result::results/success/error`, `D1ExecResult::count`, `Error::D1(e).cause()`, `IntoFuture` on statements. Needs the D1 declarations (verbatim is fine, no generics) plus the serde/`query!` layer and a typed D1 error (workers-rs-2/4). |
| `r2.rs` | **skipped** | `Bucket::list().limit/cursor/prefix/include`, `get().only_if(Conditional)`, `put(Data::{Empty,Bytes,Stream}).http_metadata/custom_metadata/md5`, `create_multipart_upload/upload_part/complete`, `delete/delete_multiple`, `Object::body().text/bytes`, `HttpMetadata`, `Include`, `FixedLengthStream::wrap`. Needs the R2 declarations and a `ByteStream`/`FixedLengthStream` body story (workers-rs-4). |
| `send_email.rs` | **skipped** | `worker::email::*` (`EmailMessage::new(_with_readable_stream)`, `SendEmail::send/send_with_builder`, `SendEmailBuilder`, `EmailAddress`, `EmailAttachment::new_attachment_with_typed_array`). `types/email.d.ts` already generates this for `worker`; it needs folding into `worker.d.ts` with `--export cloudflare:email` (workers-rs-3). |
| `container.rs` | not ported | self-skipping spec (`TEST_CONTAINER_NAME`). Needs `State::container`, `Container::running/start/get_tcp_port`, `Fetcher::fetch_request`, websocket relay. |
| `router.rs` (`/`, `/async`, OPTIONS catchall, `/init-called`, `/test-panic|abort|oom`, `/got-close-event`, catchall) | ported | `Router`, `#[event(start)]` ×2 (one `#[wasm_bindgen(start)]` calls both), `respond_with_errors` (hand-written in `fetch_handler`). Panic/abort/OOM routes exist but their spec is skipped: recovery is shim behaviour. |

Spec files: `request`, `subrequest`, `clone`, `cache`, `snippets`, `service-bindings`,
`kv`, `secret_store`, `rate_limit`, `assets`, `analytics_engine`, `queue`,
`durable`, `auto_response`, `sqlite`, `sql_iterator`, `websocket`, `socket`,
`worker` → green. `d1`, `r2`, `send_email`, `panic`, `signal` → skipped. `container`
→ self-skipped.

## Decisions

- **Non-generic binding types in `worker.d.ts`.** ts-gen does not apply default
  type arguments: a bare `KVNamespace` reference from `env.d.ts` renders as
  `KVNamespace` while the generic declaration needs `KVNamespace<JsString>`, and
  the type-alias form of `KVNamespaceListResult<Metadata, Key>` leaves `Key`/
  `Metadata` unresolved. The generics (`KVNamespace<Key extends string = string>`,
  `DurableObjectStorage.get<T>`, `Queue<Body>`, …) were dropped to `string`/`any`
  so the members stay verbatim. workers-rs-3 will hit this head-on with the real
  input; the ts-gen feature needed is default-type-argument substitution.
- **`Fetcher` / `DurableObjectStub` as plain abstract classes.** The real
  declarations are conditional types over `Rpc.*`; `DurableObjectStub extends
  Fetcher` with `id`/`name` is what workerd exposes for non-RPC use.
- **Per-type `--external` for the Env file.** The alternative (`--input` both
  files, `--export env.d.ts`) emits bare idents with no `use`, so `env.rs` does
  not compile on its own. ts-gen has no file-level external (`<file>=::crate`);
  that would collapse the seven `--external` flags in the chompfile to one.
- **Extra `constructor(body: null, init?: ResponseInit)` overload on
  `Response`.** ts-gen drops the `null` arm of `body?: BodyInit | null`, so
  `new Response(null, init)` (101 / 204) has no binding. The explicit overload
  yields `new_with_null_and_init(&Null::NULL, &init)`. Flagged as a ts-gen
  policy question below.
- **`worker.spec.ts` edits.** `type: "worker"` removed (pinned miniflare rejects
  it); `/fetch` → `/fetch-upstream`; the 404 test now only checks the status
  since the suite's catchall proxies `https://github.com/404`.
- **Test-local helpers instead of crate impls.** `util.rs` (`Result`, `err`,
  `err_message`, `url`/`path`, `ok`/`error`/`from_json`/`from_bytes`/
  `from_websocket`, `sleep`, `nullable`, `storage_get/put`,
  `block_concurrency_while`, websocket event listeners) and `sql.rs`
  (`SqlStorageValue`, cursor iteration). These are the `worker` ergonomics the
  suite relies on; they belong in `ts-gen-worker/src/ext/` (workers-rs-4) and are
  kept here so this branch does not collide with that work.
- `#![allow(clippy::all, non_upper_case_globals)]` on the crate and
  `#[allow(clippy::all)] mod env;` in the test: generated code (`pub static
  caches`, `Env::new` with 28 args) is not lint-clean by design, matching how
  ts-gen's own integration crate is configured.

## Open problems

1. **`Promise<T | null>` → `JsOption<T>` is wrong for `null`.** `JsOption` is
   empty only for `undefined` (`wasm-bindgen/src/sys.rs`), but KV `get`,
   `Cache.match`, `getAlarm`, … resolve `null`; `into_option()` yields
   `Some(null)`. Proposal: either ts-gen lowers `T | null` promise results to a
   nullish-aware wrapper (or `Option<T>` via a `catch`-style shim), or
   `JsOption::is_empty` becomes `is_null() || is_undefined()`. Until decided the
   port uses `util::nullable`. Owner: ts-gen / wasm-bindgen (Guy).
2. **Overloads with different return types collapse to the first.**
   `get(key): Promise<string|null>` / `get(key, "arrayBuffer"): Promise<ArrayBuffer|null>`
   / `get(keys: string[]): Promise<Map<…>>` all emit `JsOption<JsString>`;
   `DurableObjectStorage.get(keys[])` emits the first overload's `JsValue`. The
   port `unchecked_into`s. Proposal: key return types by expanded signature, not
   by JS name. Blocks faithful KV/DO/D1 bindings (workers-rs-3).
3. **No sync `Promise`-returning form of async bindings.** The `async fn`
   import only calls JS when polled, so `blockConcurrencyWhile` in a DO
   constructor, `ctx.waitUntil(promise)` composition, or racing with a signal
   cannot start the call synchronously. Proposal: emit a `<name>_promise()`
   companion (or make the primary return `Promise<T>` and rely on `IntoFuture`).
   Constructor gating currently passes only because the microtask beats the first
   event.
4. **`null` arm dropped in argument position** loses `f(null, later)`
   (`new Response(null, init)`). Proposal: when a nullable param is followed by
   further params, keep a `_with_null` form (ts-gen already has `Null`).
5. **Dynamic binding lookup.** `env.secret(&name)`, `env.durable_object("X")`
   returning `Err` for unknown names, and `#[durable_object]`/`#[event]` wiring
   (`start` ×N, `respond_with_errors`, panic/abort recovery, `signal`) are shim
   or macro behaviours. Which of these ts-gen-worker reproduces (and where the
   shim lives for `--target module`) needs a call before `panic.spec.ts` and
   `signal.spec.ts` can run.

## Gaps found (for routing)

- ts-gen: default type arguments not substituted (`KVNamespace` bare ref;
  alias generics `KVNamespaceListResult<Metadata, Key>` unresolved).
- ts-gen: `File | string | null` return erases to `JsValue` rather than a
  dynamic-union enum (nullable wrapper defeats the synthesis); same for
  `(ArrayBuffer | string | null)[]` elements.
- ts-gen: `Response.json()` instance method renders as `json_2` because the
  static `json` claimed the name; `worker` exposes both as `json`.
- ts-gen: `{done?: false; value} | {done: true}` inline unions emit
  `NextReturnKind { Object, Object2 }` — identical variants; a
  literal-discriminated inline object union should collapse or synthesise one
  dictionary.
- ts-gen: `declare const caches` emits `pub static caches` (needs
  `non_upper_case_globals` allowed by the consumer); consider emitting the allow.
- ts-gen: no file-level `--external` (`<file>=::crate::path`).
- ts-gen: `(...args: any[]) => void` lowers to `Function<fn(Array) ->
  Undefined>`; `Promise::new`'s `resolve` has to be `unchecked_ref`'d into it.
- crate (workers-rs-4): everything in `test/src/util.rs` and `test/src/sql.rs`;
  `Request::path()`, `Response::ok/error/from_json/from_bytes/from_websocket`,
  `Delay`, `Storage::get/put` serde wrappers, `block_concurrency_while`,
  `WebSocket::events()` stream, `FormEntry`, `SqlStorageValue`,
  `CacheDeletionOutcome`, `WebSocket::connect`.
- crate (workers-rs-4): streaming bodies (`Response::from_stream`,
  `Request::stream`, `FixedLengthStream::wrap`) — the port uses raw
  `TransformStream` + `spawn_local`.
- crate (workers-rs-2): `Error` with `Display` giving `Name: message` for JS
  errors (the `AbortError` assertions depend on it) and `From<serde_*>`.
- declarations (workers-rs-3): D1, R2, `cloudflare:email`, `Container`,
  `URLSearchParams`, `IncomingRequestCfProperties` in full.
- harness: `queue › batch send` is timing-sensitive under full-suite load (as
  upstream).
