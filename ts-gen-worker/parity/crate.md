# crate: `#[event]` macros and the `impl` layer

Owner: workers-rs-4. Branch `gbedford/tgw-crate` (on `gbedford/ts-gen-worker` `77390bf`);
ts-gen branch `agent/crate` (one unlanded commit, `462937b` on `349056c`).

Everything below is an `impl` on a generated type, a Rust-native helper type, or a
proc-macro; no wrapper structs. The layer is written against the `--fallible` bindings and
`crate::Error`/`Result<T>` from the error model; the placeholder error it started with is
gone.

## Done

- `.d.ts` additions, copied from `@cloudflare/workers-types`: `Headers.getAll/entries/keys/values`
  (the three iterators marked `@throws {never}`), `ScheduledController`,
  `ForwardableEmailMessage`, `URL`. The queue, body and Durable Object declarations come from
  the test-suite port.
- `ts-gen-worker-macros`: `#[event(fetch|scheduled|queue|email|start)]`, `respond_with_errors`,
  `#[durable_object]`, re-exported from `ts_gen_worker`.
- `src/error.rs`: `From` impls for `serde_wasm_bindgen::Error`, `serde_urlencoded::de::Error`
  (feature `serde`) and `url::ParseError` (feature `url`) on the shared `Error`.
- `src/ext/context.rs`: `ExecutionContext::spawn(impl Future<Output = ()>) -> Result<()>`.
- `src/ext/http.rs`: `Method` (`FromStr`/`Display`/`AsRef<str>`/`all()`), `Request::path()`,
  `Request::query<Q>()` (feature `serde`), `Request::url_parsed()` (feature `url`).
- `src/ext/headers.rs`: `IntoIterator for &Headers`, `FromIterator<(K, V)>`, `Default`,
  `From<HeaderMap>`/`Into<HeaderMap>` (feature `http`). `entries/keys/values/get_all` are generated.
- `src/ext/serde.rs` (feature `serde`, default as in `worker`): `Request::json_into<T>()`,
  `Response::json_into<T>()`, `ResponseInitBuilder::from_json(&T)`, `Message::body_into<T>()`.
- `src/ext/streams.rs`: `ByteStream` (+ `empty()`, `into_inner()`, `http_body::Body` under
  `http`), `Request::bytes_stream()`, `Response::bytes_stream()`,
  `ResponseInitBuilder::from_stream(S)`, `readable_stream_from(S)`.
- `src/ext/handler.rs`: `FromRequest`/`IntoResponse`, the `#[event(fetch)]` conversion
  traits. `src/ext/interop.rs` (feature `http`): `HttpRequest`/`HttpResponse`, the four
  `TryFrom` impls, `From<http::Error>`.
- `src/ext/response.rs`: `Response::builder()` → the generated `ResponseInitBuilder`, carrying
  `ok`, `error`, `empty`, `from_html`, `from_bytes` (+ `from_json`, `from_stream` above).
- `src/router.rs`: `Router<'a, E, D = ()>` / `RouteContext<E, D>` over the generated types.
- `src/durable.rs`: `DurableObject` trait with `type Env`, `fetch`, `alarm`,
  `websocket_message/close/error`, and the `has_durable_object_attribute` guard.
- ts-gen `a643100` (agent/crate): type-parameter defaults land on the `pub type`
  declaration (`MessageBatch<Body: JsGeneric = JsValue>`), so a defaulted generic is a
  complete bare name, as is the constructor return of a class whose parameters all default.
  Fixture, integration compile test, CONVENTIONS section. No output change for this crate
  today: the port's declarations drop generics; the real workers-types input
  (`ExecutionContext<Props = unknown>`, `MessageBatch<Body = unknown>`) needs it.
- Tests, append-only in the ported tree: `test/src/ext.rs` (routes under `/ext/*`, a
  `MacroCounter` `#[durable_object]`, `scheduled`/`email` handlers, a second `start`), one
  arm in `lib.rs`'s dispatch, one `env.d.ts` line, and `test/tests/ext.spec.ts` with its own
  Miniflare instance (`unsafeTriggerHandlers`): 15 cases, one per feature. Suite: 176
  passed, 6 skipped (161 before).

Session 2 (port adoption, the migration dry run), one commit per item, 176 green after each:

- `Error` `Display` (`cab5b36`): `Js` renders `Name: message` for an `Error` instance;
  a thrown primitive renders as `String(v)` through ts-gen `e79bdee` (agent/crate), which
  calls the global `String()` from the generated `JsException` `Display` with a `catch`
  import so an object without `toString` still falls back to `Debug`. `respond_with_errors`
  takes the body from the handler error's own `Display` (bound `Display + Into<JsValue>`
  in that mode, as the plan said; the default mode keeps `Into<JsValue>` alone).
- `#[event(fetch, respond_with_errors)]` replaces the port's `fetch_handler`; `util::err_message`
  deleted (21 call sites → `e.to_string()`).
- Two `#[event(start)]` replace the chained `#[wasm_bindgen(start)]`.
- `#[event(queue)]` + `message.body_into()?` + `req.json_into()` replace the `queue` export.
- `DurableObject` grows `websocket_message/close/error` and the export guard; `#[durable_object]`
  exports them by default, `(websocket)` selects them. Then every DO class in the port
  (`Counter`, `PutRawTestObject`, `AutoResponseObject`, `MyClass`, `AnotherClass`, `SqlCounter`,
  `SqlIterator`, `AlarmObject`) is `#[durable_object]` + `impl DurableObject`.
- `util::ok/error/from_json/from_bytes/from_websocket/url/path` deleted (≈220 call sites →
  `Response::builder().*`, `req.url_parsed()`, `req.path()`; the test crate enables feature
  `url`). `util.rs` is 212 → 147 lines; what remains is the gap list below.
- `http` feature (`1d05f1e`): `Request`/`Response` ↔ `http::Request`/`http::Response<B>`
  as `TryFrom` with `ByteStream` as the body, `HttpRequest`/`HttpResponse` aliases,
  `FromRequest`/`IntoResponse` so `#[event(fetch)]` accepts either signature,
  `chomp test:ts-gen-worker-http` (+ CI step) runs the whole suite with every request and
  response round-tripped through the `http` types: 176 green in both modes.

Not adopted: `Router` for `route()` (decision below). `connect` export → `#[event(connect)]`
done in Session 3.

## Decisions

- **Handler wrapper placement.** `#[event]` leaves the user function untouched and emits the
  wasm-bindgen export inside `const _: () = { ... }` next to it, with the parameter types
  copied verbatim (except `fetch`'s request, see `FromRequest` below). Block scope sees the module's items and imports, so the app's generated
  `Env` resolves without the macro knowing its path, and nothing new enters the module
  namespace. Unlike `worker-macros` there is no renaming of the user fn.
- **Error bound is `Into<JsValue>`; `respond_with_errors` adds `Display`.** The first
  version took the body text from `Error::from(js).to_string()` with one bound, but once
  `Error::Js` displays `Name: message` a `Rust("boom")` error round-tripped through
  `JsValue` would read `Error: boom`. The macro now formats the handler's own error before
  converting it, so handlers returning `Result<_, JsValue>` work in the default mode only.
  The error is logged via `console.error` as the JS value, so JS errors keep their stacks in
  logs. `respond_with_errors` on any other kind is a compile error (`worker` ignored it).
- **`Error::Js` displays `Name: message`** for an `Error` instance (master decision): the
  `worker` format the migrating specs assert on (`"Error: Cannot store value"`,
  `"AbortError"`). The generated `JsException` `Display` stays message-only; the crate's
  `Error` adds the name, so a user matching `JsException` directly sees the message.
- **`websocket_message` takes the generated `DataKind`** (`string | ArrayBuffer`), the type
  `MessageEvent::data()` already returns, rather than a second `WebSocketIncomingMessage`
  enum: one type for a websocket message whether it arrives through the hibernation
  callback or an event listener. `close`/`error` keep `worker`'s shapes (`usize`, `String`,
  `bool`; `Error`). The name is ts-gen's synthesized anchor and may move with the real
  input; the trait is the only place that spells it.
- **Export guard** is `worker`'s trick verbatim: `DurableObject: has_durable_object_attribute`,
  a hidden marker only `#[durable_object]` implements, so the missing attribute is named in
  the compile error. Re-exported through `__macro`.
- **`route()` stays a slice match, not `Router`.** The port's handlers take
  `(req, env, data)`; `worker`'s original went through `Router` only via a `handler!` macro
  adapting that signature to `(req, RouteContext)`. Moving to `Router` would re-add the
  adapter layer for no coverage gain (`ext.spec.ts` covers `Router` directly: sync, async,
  params, 404, 405) and the `http` round trip is simpler with one `route(Request)` entry.
- **`ByteStream` is the `http` body**, not a separate `Body`. It gained an empty variant
  (`ByteStream::empty()`, `into_inner() -> Option<ReadableStream>`) and implements
  `http_body::Body<Data = Bytes>` under the feature, so `http::Request<ByteStream>` is
  `worker`'s `http::Request<Body>` with the stream type the layer already had.
- **`#[event(fetch)]` always exports a `Request`-taking wrapper** and converts through
  `FromRequest`/`IntoResponse` (identity for the generated types; `TryFrom` for the `http`
  ones), as `worker` does. The handler's `env`/`ctx` types stay verbatim. Conversion errors
  go through the same error policy as handler errors.
- **Extensions carry the generated types directly**: `AbortSignal` and
  `IncomingRequestCfProperties` on `http::Request`, `WebSocket` on `http::Response`. No
  `Cf`/`RequestRedirect` wrappers: this wasm-bindgen's `JsValue` is `Send + Sync` off
  `atomics`, so the generated handles satisfy `http::Extensions` as they are. `redirect`
  and response `cf` are not carried (`Request.redirect` is not declared in the port's
  `.d.ts`; `Response.cf` is an untyped `JsValue`).
- **`queue`/`email` failures reject** rather than panic as `worker` does. A rejected promise
  is what makes the runtime retry the batch / bounce the message; a trap does the same by
  accident and loses the message text.
- **`scheduled` returns `()`** as in `worker`.
- **`ExecutionContext::spawn(fut) -> Result<()>`** rather than a Future-taking `wait_until`.
  The generated `wait_until(&Promise)` keeps its JS name and type (design rule); a Future is
  not a Promise, so the Rust form gets the vocabulary of `wasm_bindgen_futures::spawn_local`,
  with the context-bound lifetime as the one difference. It returns `Result` because
  `wait_until` is a `--fallible` binding; `worker`'s returned `()`.
- **`Response::builder()` is the home of every `worker` constructor shortcut.** Reasons, in
  order: (1) `Response::ok(body)` cannot exist, the generated instance getter `ok()` already
  takes the name; (2) `Response::error(msg, status)` would collide with the generated static
  `Response.error()` once the real workers-types input lands; (3) `worker`'s `with_status` /
  `with_headers` cannot exist on an immutable JS `Response` without reconstructing it (which
  locks the body, drops `webSocket`/`cf`, and throws for null-body statuses). The generated
  `ResponseInitBuilder` is exactly `worker`'s `ResponseBuilder`, so the shortcuts become
  terminal methods on it: `Response::builder().status(201.0).ok("x")`. One rule, no second
  builder type; `web_socket`/`cf`/`encode_body` are already on the builder from the port's
  `ResponseInit`. `ok`/`from_html` set `content-type` only if absent (`worker` overwrote it,
  which made `with_headers` ordering matter).
- **`json_into`** for deserialising `Request`/`Response` bodies: the real input declares
  `json<T>(): Promise<T>` on both, which ts-gen lowers as `json<T: JsGeneric>` (and `json_2`
  on `Response` beside the static). Parsing goes through `text()` + `serde_json`, giving exact
  serde integer semantics; serialisation goes through `Response.json()` with
  `serde_wasm_bindgen::Serializer::json_compatible()` so maps are objects and the platform
  sets `content-type`.
- **`Method` parses strictly** (`FromStr`, unknown method is an error); `worker` mapped unknown
  methods to `GET`. Rather than add `Request::method_enum()`, the generated `method() -> String`
  stays the accessor and `req.method().parse::<Method>()?` is the conversion. The router
  treats an unparsable method as no match.
- **`Router<'a, E, D = ()>`** with the env type first. `Env` is user-generated so the router
  must be generic over it, and closure parameter inference needs `E` known before the first
  `.get(..)`: `Router::<Env>::new()`. `D` keeps its default. `RouteContext` drops `worker`'s
  binding helpers (`kv`, `secret`, ...); `ctx.env` is the app's typed `Env`.
- **`Request::path()`/`query()` use the platform `URL`** (declared verbatim) instead of the
  `url` crate; `url_parsed()` is behind the `url` feature for callers who want `url::Url`.
- **Durable Objects: implemented.** `#[durable_object]` only needs the `#[wasm_bindgen]` class
  export plus the generated `DurableObjectState`. The user-generated `Env` is reached as the
  associated type `<T as DurableObject>::Env`, which wasm-bindgen accepts in the exported
  constructor signature. `alarm` returns `Result<()>` (the JS handler is `void`), not
  `worker`'s `Result<Response>`.
- **`FromIterator for Headers` rethrows** an invalid header as the JS `TypeError`
  (`wasm_bindgen::throw_val`): `collect()` has no error channel, `worker` unwrapped, and
  rethrowing is exactly what the pre-`--fallible` binding did.
- **`worker.d.ts` merge with the port.** Where both sides declared a type (`Message`,
  `MessageBatch`, `QueueRetryOptions`, `Request/Response.bytes`, the DO family), the port's
  declaration stands: it is member-verbatim and exercised by 161 tests, where this branch's
  DO subset was simplified. Only the four declarations the port lacked were kept.
- **Tests are append-only** in the ported tree: a new module, a new spec file, one dispatch
  arm, one `env.d.ts` line. `worker.spec.ts` and `lib.rs`'s existing arms are untouched.

## Open problems

- **`http` round trip double-wraps streaming bodies.** `TryFrom<http::Request<B>>` is
  generic over `B`, so a `ByteStream` body goes Rust stream → JS `ReadableStream` → Rust
  again rather than handing back the original `ReadableStream` (`into_inner()`). Same as
  `worker`; a non-generic fast path would need specialization or a separate `From`.
- **`Headers` ↔ `HeaderMap` drops invalid entries silently** (pre-existing in
  `ext/headers.rs`); `worker` errored. Affects the `http` conversions only.

## Gaps found

- ts-gen: a generic type alias that erases (`type Loose<T> = T | string`) emits
  `pub type Loose<T> = JsValue` (E0091, unused `T`) and a clashing `use JsValue as Loose`;
  `CfProperties<HostMetadata>` in workers-types is one. Pre-existing; found while adding
  the defaults fixture, which avoids it.
- ts-gen: a generic class with a non-defaulted parameter still declares its constructor as
  returning the bare name (`new() -> Result<G, JsValue>` for `class G<T>`), which does not
  compile. Pre-existing; the defaults fix only covers fully defaulted classes.
- ts-gen: the generic `blockConcurrencyWhile<T>(callback: () => Promise<T>): Promise<T>`
  generates code that does not compile (`Function<fn() -> Promise<T>>` with `T: JsGeneric`
  fails `ErasableGeneric`/`FromWasmAbi` bounds); the port declares it with `Promise<any>`.
- Env generation: an app `env.d.ts` referencing crate types needs
  `--external "X=::ts_gen_worker::X"` per type (eleven in the chompfile now). A fallback
  mapping for unresolved globals to a crate path would remove that.
- Crate-layer idioms still local to the port: see Session 3 "Gaps found" for the current
  `util.rs` list (the `@rust` integer hints, native awaited strings, `console_log!`,
  `Delay`, `#[event(connect)]`, `into_parts` and the storage serde pair since landed).
- `Request.redirect` is not declared in the port's `worker.d.ts` (workers-types has
  `readonly redirect: string`), so the `http` conversion cannot carry it.


## Session 3: migration friction pass

Every row of the table below was re-verified by compiling a scratch route in
`test/src/ext.rs` holding each right-hand side (removed again; the feature routes stay).
`parity/test-port.md`'s Session 2 rows are merged in, so this is the one table. Suite:
192 passed, 4 skipped in both `test:ts-gen-worker` and `test:ts-gen-worker-http` after
the rebase onto `77390bf` (185 before the session).

### Done

- ts-gen `462937b` (agent/crate): every dictionary builder implements `From<Dict>`, so a
  helper can `build()`, read/modify through the generated getters and setters, and
  `.into()` back to resume the chain. Fixtures re-blessed, CONVENTIONS section. Closes
  workers-rs-5's "continue a builder from a dictionary" gap.
- `#[event(connect)]`: `async fn(Socket, Env, ExecutionContext) -> Result<(), E>`;
  rejects on error, closing only that socket. `socket.rs` adopts it (`socket.spec.ts`
  covers TCP and UDP inbound).
- `Response::into_parts() -> (ResponseInitBuilder, Option<ReadableStream>)` and
  `ResponseInitBuilder::body(Option<ReadableStream>)`; the port's `catchall` adopts them.
- `ResponseInitBuilder::header(k, v)?`, `cors(&Cors)?`, `from_websocket(ws)`;
  `Request::new_with_method(url, Method)`; `DurableObjectStorage::get_into::<T>()` /
  `put_serialized(k, &v)` (feature `serde`). The port adopts each where the semantics are
  identical (`text` helper, `respond*`, the three websocket upgrades, `fetch.rs`, every
  DO class); `util.rs` loses `storage_get`/`storage_put` (123 → 93 lines).
- `with_cf` / `with_encode_body` / `with_websocket`: the generated builder already has
  `cf(&JsValue)`, `encode_body("manual")`, `web_socket(&ws)`; documented, exercised in
  `ext.spec.ts` (`builder-header`, `into-parts`).
- `MIGRATING.md`: the user-facing guide distilled from the table.

### Decisions

- **`connect` has no protocol option.** `worker`'s `connect(udp)` produced a second
  `connect_udp` export that worker-build's shim merged into one `connect`; without the
  shim there is one export, so one handler serves both protocols and dispatches on the
  socket's protocol. `#[event(connect(udp))]` is a compile error saying so.
- **`into_parts` re-constructs.** A JS `Response` is immutable, so the builder comes back
  loaded with `status`, `headers`, `webSocket` and `cf` (when set) and the body is the
  stream; `statusText` is not carried because the platform derives the default from the
  final status, and carrying the upstream phrase across a status change would be wrong.
  `body(None)` is `empty()`. The `clone()`-based alternative was rejected: it cannot
  change the status either.
- **`header`/`cors` live on the builder with field-setter names** (`status`, `headers`,
  `cf` are the builder's vocabulary), not `with_*`; `header` uses `set` as `worker` did.
  They read the headers set so far by normalizing `init.headers()` through `new
  Headers(init)`, so a preceding `headers_with_record` works too.
- **`from_websocket(ws)` takes the socket by value**, as `worker::Response::from_websocket`
  did, for a verbatim move to `Response::builder().from_websocket(ws)`.
- **`Request::new_with_method`** takes the overload-style name because the generated
  `Request::new(&Request)` owns the JS constructor name; it takes `Method`, not `&str`,
  to keep `worker`'s typed form. The general fix (a builder accepted where the dictionary
  is expected, so `new_with_str_and_init(url, &RequestInit::builder().method("POST"))`
  needs no `build()`) is a ts-gen question recorded below.
- **Storage serde names.** `get_into::<T>` follows `json_into`/`body_into` (deserialize
  into `T`); the write side is `put_serialized(k, &v)` rather than `put_into` (reads as the
  wrong direction) or `put_from`. Serialization is plain `serde_wasm_bindgen::to_value`,
  not `json_compatible()`, since storage is structured clone and `worker` stored maps as
  `Map`s.
- **Friction rows chosen.** By ratio of column lengths: `Request::new(url, Method)` (2.7×),
  `.with_header` (2.2×), `Response::from_websocket` (2×), `ResponseBuilder::with_cors`
  (2×), and `into_parts`/`body` (not ported). `Fetch::Url(..).send()`, `send_with_signal`,
  `console_log!`, `text()`, `status_code()` had already become "same" through
  workers-rs-5 and the native awaited values; the storage serde pair was the one
  remaining `worker` idiom still local to the port's `util.rs` that is an `impl`.

### Open problems

- `Response.cf` deserialization is `serde_wasm_bindgen::from_value(res.cf())?`; a
  `cf_into::<T>()` would mirror `worker::Response::cf::<T>()` but `cf` is `any` upstream, so
  it was left as the one-liner. Say if it should be added.

### Gaps found

- ts-gen: a `&FooBuilder` is not accepted where `&Foo` is expected, so every inline
  dictionary needs `.build()` (`RequestInit::builder().method("POST").build()`). `impl
  AsRef<Foo> for FooBuilder` or `Deref` would remove it; the cheapest is `Deref<Target =
  Foo>` with the setter/getter name clash noted (the builder's `headers(self, v)` shadows
  the dictionary's getter).
- `Headers::keys()`/`values()` return `Iterator<JsString>` whose items are
  `Result<JsString>`; a `String` item lowering would make `for k in headers.keys()`
  direct.
- Remaining `util.rs` (93 lines): `sleep` (→ `Delay`, trivial), `websocket_events` /
  `on_websocket_events` / `WebsocketEvent` / `message_text` (`WebSocket::events()`),
  `block_concurrency_while` over a Rust future (blocked on the generic
  `blockConcurrencyWhile<T>` ts-gen gap).

## Migration table

The non-binding `worker` surface used in `test/src/*.rs` (plus the handler kinds and context
calls it does not use), and its `ts-gen-worker` form, as the code is after Session 3. Rows from
`parity/test-port.md` Session 2 are merged in. Binding APIs beyond KV and DO storage (R2, D1,
Cache, queue producers, SQL storage, sockets, rate limiting, analytics, secrets store,
containers, assets, `send_email`) are not listed: `parity/test-port.md` has them module by
module. Under `--fallible` every sync binding that can throw returns `Result`, so `worker`'s
`?` stays where it was and appears on a few calls that had none.

| `worker` | `ts-gen-worker` |
| --- | --- |
| **Handlers** | |
| `#[event(fetch)] async fn f(req: Request, env: Env, ctx: Context)` | same, with `ctx: ExecutionContext` and `env` the app's type generated from its `env.d.ts` |
| `#[event(fetch, respond_with_errors)]` | same |
| `#[event(fetch)] async fn f(req: HttpRequest, ..) -> Result<http::Response<B>>` (feature `http`) | same; `HttpRequest = http::Request<ByteStream>`, `HttpResponse = http::Response<ByteStream>` |
| `worker::Body` / `Body::empty()` / `body.into_inner()` | `ByteStream` / `ByteStream::empty()` / `into_inner()` |
| `http::Request<Body>: TryFrom<Request>` and back; same for `Response` | same (`try_into()`); extensions `AbortSignal`, `IncomingRequestCfProperties`, `WebSocket` instead of `Cf`/`RequestRedirect`/`WebSocket` |
| `#[event(start)]` (several allowed) | same |
| `#[event(queue)] async fn q(batch: MessageBatch<T>, ..) -> Result<()>` | `batch: MessageBatch`; `message.body_into::<T>()?` |
| `#[event(scheduled)] async fn s(event: ScheduledEvent, env, ctx: ScheduleContext)` | `(controller: ScheduledController, env, ctx: ExecutionContext)` |
| `#[event(email)]` | same |
| `#[event(connect)] async fn c(socket: Socket, env, ctx)` | same |
| `#[event(connect(udp))] async fn c(socket: UdpSocket, ..)` | one `connect` handler for both protocols, dispatching on the socket's protocol |
| `#[send]`, `send::SendFuture`, `send::SendWrapper`, `IntoSendFuture` | same |
| `ctx.wait_until(fut)` | `ctx.spawn(fut)?` |
| `ctx.pass_through_on_exception()` | `ctx.pass_through_on_exception()?` |
| **Durable Objects** | |
| `#[durable_object] struct S; impl DurableObject for S { fn new(state: State, env: Env) }` | same, plus `type Env = Env;`, `state: DurableObjectState` |
| `async fn alarm(&self) -> Result<Response>` | `-> Result<()>` |
| `async fn websocket_message(&self, ws, msg: WebSocketIncomingMessage)` | `msg: DataKind` (`String(s)` / `ArrayBuffer(buf)`) |
| `async fn websocket_close(&self, ws, code: usize, reason: String, was_clean: bool)` / `websocket_error(&self, ws, err: Error)` | same |
| `#[durable_object(fetch | alarm | websocket)]` | same (`connect` not ported) |
| `impl DurableObject` without `#[durable_object]` | compile error naming `has_durable_object_attribute`, as `worker` |
| `env.durable_object("X")?.id_from_name(n)?.get_stub()?` | `env.x().get_by_name(n)?` (or `env.x().get(&env.x().id_from_name(n)?)?`) |
| `stub.fetch_with_str(url).await?` | same |
| `stub.fetch_with_request(req).await?` | `stub.fetch(&req).await?` |
| `state.storage().get::<T>(k).await?` → `Option<T>` | `storage.get_into::<T>(k).await?` (`storage.get::<JsValue>(k)` for the raw handle) |
| `state.storage().put(k, v).await?` | `storage.put_serialized(k, &v).await?` (`storage.put(k, &js_value)` for a handle) |
| `state.storage().delete(k)` / `list()` / `get_multiple` | `delete(k)` / `list()` → `Map` / `get_with_keys(&[..])` → `Map` |
| `state.block_concurrency_while(fut)` | not ported (`util::block_concurrency_while`; generic `blockConcurrencyWhile<T>` is a ts-gen gap) |
| **Response** | |
| `Response::ok(body)` | `Response::builder().ok(body)` |
| `Response::error(msg, status)` | `Response::builder().error(msg, status)` |
| `Response::from_json(&v)` | `Response::builder().from_json(&v)` |
| `Response::from_bytes(v)` | `Response::builder().from_bytes(v)` |
| `Response::from_html(s)` | `Response::builder().from_html(s)` |
| `Response::from_stream(s)` | `Response::builder().from_stream(s)` |
| `Response::empty()` | `Response::builder().empty()` |
| `Response::from_websocket(ws)` | `Response::builder().from_websocket(ws)` |
| `Response::redirect(url)` / `redirect_with_status(url, 302)` | `Response::redirect(url.as_str())?` / `Response::redirect_with_status(url.as_str(), 302)?` |
| `Response::builder()` / `ResponseBuilder::new()` | `Response::builder()` (the generated `ResponseInitBuilder`) |
| `.with_status(201)` | `.status(201)` on the builder, before the terminal call |
| `.with_headers(h)` | `.headers(&h)` |
| `.with_header(k, v)?` | `.header(k, v)?` |
| `.with_cors(&cors)?` on the builder | `.cors(&cors)?` |
| `.with_websocket(ws)` | `.web_socket(&ws)` |
| `.with_encode_body(EncodeBody::Manual)` | `.encode_body("manual")` |
| `.with_cf(v)?` | `.cf(&serde_wasm_bindgen::to_value(&v)?)` |
| `res.into_parts()` → `(builder, body)`; `builder.with_status(404).body(body)` | `res.into_parts()`; `builder.status(404).body(body)?` |
| `res.status_code()` | `res.status()` |
| `res.websocket()` | `res.web_socket()` |
| `res.cf::<T>()?` | `serde_wasm_bindgen::from_value(res.cf())?` |
| `res.encode_body()` | not readable from a JS `Response` |
| `res.text().await?` | same |
| `res.json::<T>().await?` | `res.json_into::<T>().await?` |
| `res.bytes().await?` | `res.bytes().await?.to_vec()` |
| `res.stream()?` | `res.bytes_stream()` (`Option`) |
| `res.cloned()?` | `res.clone()?` |
| `res.headers()` / `res.headers_mut()` | `res.headers()` (JS `Headers` is mutable; immutable-guard errors on a fetched response) |
| `res.with_cors(&cors)?` | same |
| **Request** | |
| `Request::new(url, Method::Post)` | `Request::new_with_method(url, Method::Post)` |
| `Request::new_with_init(url, &init)` | `Request::new_with_str_and_init(url, &init)` |
| `RequestInit::new().with_method(..)` / `.with_body(Some(v))` / `.with_headers(h)` | `RequestInit::builder().method(..)` / `.body_with_str(..)` / `.headers(&h)`, then `.build()` |
| `Fetch::Url(url).send().await?` | `Fetch::Url(url.into()).send().await?` (or `fetch_with_str(url).await?`) |
| `Fetch::Request(req).send().await?` | same (or `fetch(&req).await?`) |
| `fetch.send_with_signal(&signal)` / `AbortController::default()` | same |
| `req.method()` → `Method` | `req.method()` → `String`; `req.method().parse::<Method>()?` |
| `req.path()` | same |
| `req.url()?` → `url::Url` | `req.url()` → `String`; `req.url_parsed()?` (feature `url`) |
| `req.query::<Q>()?` | same |
| `req.text().await?` | same |
| `req.json::<T>().await?` | `req.json_into::<T>().await?` |
| `req.bytes().await?` | `req.bytes().await?.to_vec()` |
| `req.stream()?` | `req.bytes_stream()` |
| `req.form_data().await?` | same (`FormData.get` is a `JsValue`) |
| `req.headers()` / `req.headers_mut()?` | `req.headers()` |
| `req.cf()` | `req.cf()` (`Option<IncomingRequestCfProperties>`) |
| `req.clone()?` | same |
| **Headers** | |
| `Headers::new()` | `Headers::new()?` or `Headers::default()` |
| `headers.get(k)?` / `has(k)?` / `set(k, v)?` / `append(k, v)?` / `delete(k)?` | same (`JsException::TypeError` on a bad name) |
| `headers.get_all(k)?` | same (`Set-Cookie` only under workerd) |
| `headers.entries()` / `for (k, v) in &headers` | same |
| `headers.keys()` / `values()` | generated `js_sys::Iterator<JsString>` (items are `Result<JsString>`) |
| `[(k, v)].into_iter().collect::<Headers>()` | same |
| `HeaderMap` ↔ `Headers` (feature `http`) | `From` both ways |
| **Router** | |
| `Router::new()` / `Router::with_data(d)` | `Router::<Env>::new()` / `Router::<Env, D>::with_data(d)` |
| `.get/post/.._async(pattern, handler)` / `.run(req, env)` | same |
| `ctx.param("id")` / `ctx.data` / `ctx.env` | same |
| `ctx.kv("X")?` / `ctx.secret("X")?` / `ctx.durable_object("X")?` | `ctx.env.x()` (generated getters) |
| **Env** | |
| `env.kv(..)` / `bucket` / `d1` / `queue` / `service` / `durable_object` | `env.<binding>()` generated from the app's `env.d.ts` |
| `env.secret("X")?.to_string()` / `env.var("X")?.to_string()` | `env.secret("X")?` / `env.var("X")?` (`String`; `EnvExt`) |
| `env.object_var::<T>("X")?` | same (`EnvExt`) |
| `env.kv("X")?` / `env.get_binding::<T>("X")?` | `env.get::<KVNamespace>("X")?` / `env.get::<T>("X")?` (typed `env.x()` preferred) |
| **KV** | |
| `kv.get("k").text().await?` | `kv.get_builder("k").text().await?` (or `kv.get("k").await?.into_option()`) |
| `kv.get("k").json::<T>().await?` / `.bytes()` / `.stream()` / `.cache_ttl(n)` | `kv.get_builder("k").json::<T>()` / same |
| `kv.get("k").text_with_metadata::<M>().await?` (+ `json`/`bytes`/`stream` forms) | `kv.get_builder("k").text_with_metadata::<M>()` |
| `kv.get_bulk(&keys).text().await?` / `.json::<T>()` / `.text_with_metadata::<M>()` / `.json_with_metadata::<T, M>()` | same |
| `kv.put("k", "v")?.execute().await?` | `kv.put("k", "v").await?` |
| `kv.put("k", &t)?.expiration_ttl(60).metadata(m)?.execute().await?` | `kv.put_builder("k", &t)?.expiration_ttl(60).metadata(m)?.execute().await?` |
| `kv.put_bytes("k", &b)?.execute()` / `kv.put_stream("k", s)?.execute()` | `kv.put_bytes("k", &b).execute()` / `kv.put_stream("k", s).execute()` (infallible builders) |
| `kv.list().prefix(p).limit(n).cursor(c).execute().await?` | `kv.list_builder().prefix(p).limit(n).cursor(c).execute().await?` |
| `list.keys[i].name` / `.expiration: Option<u64>` / `list.list_complete` / `list.cursor` | `list.keys().get(i).name()` (`Array`) / `.expiration() -> Option<f64>` / `list.list_complete()` / `list.cursor()` |
| `kv.delete("k").await?` | same |
| `KvError` | `Error` |
| **Errors and misc** | |
| `Error::from(..)` / `Error::RustError(s)` / `Error::D1(e)` | `Error::from(s)` / `Error::Rust(s)` / `Error::Js(JsException::..)` |
| `err.to_string()` → `Name: message` for JS errors | same (`42` / `null` for a thrown primitive) |
| `Result<T, E>` / `ok::Ok(v)` | same |
| `console_log!("{}", x)` (and `debug`/`warn`/`error`) | same |
| `Delay::from(duration).await` | same |
| `Date::now()` / `Date::now().as_millis()` | `js_sys::Date::current()` (`DateExt`) / `js_sys::Date::now() as u64` |
| `Date::new(DateInit::Millis(n))` / `DateInit::Millis(n).into()` | `js_sys::Date::from_init(DateInit::Millis(n))` / same `.into()` |
| `date.as_millis()` | same (`DateExt`) |
| `Cors::new().with_origins(..)` | same |
