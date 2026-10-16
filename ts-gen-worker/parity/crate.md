# crate: `#[event]` macros and the `impl` layer

Owner: workers-rs-4. Branch `gbedford/tgw-crate` (on `gbedford/ts-gen-worker` `3f5a311`);
ts-gen branch `agent/crate` (nothing unlanded).

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

None open from Sessions 1–2; the `http` double-wrap and the lossy `Headers` ↔ `HeaderMap`
conversion were resolved in Session 4.

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

## Session 4: the two `http` open problems and a size/startup comparison

### Done

- **`http` round trip hands the original `ReadableStream` back.** `ByteStream` keeps the JS
  stream as given and locks it to a reader on the first poll, reading through the generated
  `ReadableStreamDefaultReader` bindings; `into_inner()` releases the lock and returns the
  same object with the unread chunks still in it. The conversions back to the platform
  types recognise a `ByteStream` body by downcast and hand its stream straight through.
  Test (`ext.spec.ts` "http round trip"): request and response bodies come back as the
  same object (`==` on the handle), and a partially read stream continues from the next
  chunk.
- **`Headers` ↔ `HeaderMap` is `TryFrom`.** A non-ASCII value going to JS, or a byte `http`
  accepts and the Fetch spec does not (DEL) coming back, is an error naming the header;
  the `http` interop propagates it. Test: both directions.
- The test crate always builds the `http` conversions (`ts-gen-worker/http` on the
  dependency); its own `http` feature only switches the fetch export's signature, so the
  conversions are exercised by `test:ts-gen-worker` too.
- `bench/coldstart.mjs` and the measurements below. Suite: 194 passed, 4 skipped in both
  chomp tasks (192 before).

### Decisions

- **No non-generic `From<http::Request<ByteStream>>`.** The plan asked for one beside the
  generic `TryFrom<http::Request<B>>`, but core's blanket `impl<T, U: Into<T>> TryFrom<U>
  for T` makes the two overlap for `B = ByteStream` (a coherence error), and the same holds
  for `IntoResponse`. Instead the generic impl downcasts the body (`B: 'static` already, so
  `&mut Option<B> as &mut dyn Any`) and takes the fast path for a `ByteStream`; every
  caller, including `#[event(fetch)]`, gets it with one API.
- **`ByteStream` reads through the generated reader bindings, not `wasm_streams`.**
  `wasm_streams::ReadableStream::into_stream` cancels the stream on drop, which made
  `into_inner()` after a partial read return a cancelled stream (the first test run proved
  it: the remainder was empty). The reader form that only releases the lock borrows the
  wrapper, so it cannot be stored. Reading by hand is forty lines over
  `get_reader()` / `read_promise()` / `release_lock()` / `cancel()`; `wasm_streams` stays
  for the Rust → JS direction (`readable_stream_from`).
- **Drop still cancels.** A polled `ByteStream` that is dropped cancels the JS stream, as
  `worker`'s did (a dropped body should not leave the producer waiting); only
  `into_inner()` detaches without cancelling. An unpolled `ByteStream` never locked the
  stream, so dropping it leaves the stream untouched.
- **A disturbed stream is still the same stream.** After a partial read, `into_inner()`
  returns the original object, but the platform refuses a stream that has been read from
  as a `Response` body (`TypeError: This ReadableStream is disturbed`); a streams consumer
  can continue it. Documented on `into_inner`.
- **No lossy header conversion is kept.** Nothing needed it; a caller who wants to drop
  entries can filter the `HeaderMap` first.
- **JS `Headers` flatten repeated names.** `Headers → HeaderMap` yields one `x-a: 1, 2`
  entry for two appended `x-a` headers because that is what `Headers.entries()` yields;
  only `Set-Cookie` stays separate. Not a conversion bug; noted because the test shows it.

### Measurements

Release builds of the two test workers (`test/` with `worker`, `ts-gen-worker/test` with
`ts-gen-worker`), same workspace `[profile.release]` (`opt-level = "z"`, `lto`,
`codegen-units = 1`), same wasm-bindgen, same wasm-opt (binaryen 132 from
`~/.cache/worker-build`, the feature flags `worker-build` passes). Informational; nothing
was optimized.

| | `worker` (`test/`) | `ts-gen-worker` (`ts-gen-worker/test`) |
| --- | ---: | ---: |
| `cargo build --release` `.wasm` | 14,348,579 | 13,804,798 |
| after `wasm-bindgen --target module` | 13,430,546 | 12,752,313 |
| of which code section / data section / `name` | 1,525,206 / 11,059,526 / 811,145 | 1,222,539 / 11,011,941 / 483,320 |
| after `wasm-opt -Oz` | 7,105,245 | 6,886,283 |
| of which code section / data section | 1,274,832 / 5,800,039 | 1,100,617 / 5,752,429 |
| wasm-bindgen `index.js` (+ snippets) | 139,829 (+335) | 152,642 (+239) |
| shipped by `worker-build --release` (`wasm-opt -O --debuginfo`, esbuild-minified shim + glue) | 7,621,242 + 125,906 | n/a |
| cold start to first response, median of 10 (two runs) | 207 ms, 190 ms | 194 ms, 168 ms |

The data section (≈5.7 MB after `-Oz`, a segment of mostly `0x01` bytes) is common to
both workers and dominates both sizes; it comes from the suites' shared dependencies,
not from either crate, and was not investigated. The code sections are the comparable
numbers: the generated bindings are 14% smaller than the hand-written `worker-sys` +
`worker` layer for the same suite. The generated JS glue is 9% larger (every binding is an
import with its own glue, where `worker` shares `web-sys` imports). Cold start is
dominated by starting the workerd process under Miniflare (160–280 ms per fresh
instance); the two are indistinguishable within that noise.

Commands (from the workspace root; `WB=wasm-bindgen/target/debug/wasm-bindgen`,
`OPT=~/.cache/worker-build/binaryen-x86_64-linux-version_132_jspi_hooks_1/bin/wasm-opt`,
`FEAT` = the seven `--enable-*` flags in `worker-build/src/build/mod.rs`):

```sh
cargo build --release --target wasm32-unknown-unknown -p ts-gen-worker-test
cargo build --release --target wasm32-unknown-unknown -p worker-sandbox
for p in ts_gen_worker_test:tgw worker_sandbox:worker; do w=${p%%:*}; n=${p##*:}
  $WB --target module --out-name index --force-enable-abort-handler --out-dir /tmp/bench-$n \
    target/wasm32-unknown-unknown/release/$w.wasm
  $OPT -Oz $FEAT -o /tmp/bench-$n/index_bg.oz.wasm /tmp/bench-$n/index_bg.wasm
done
(cd test && WASM_BINDGEN_BIN=../wasm-bindgen/target/debug/wasm-bindgen ../target/debug/worker-build --release)
# cold start: a dir holding index.js, index_bg.wasm (the -Oz one for ts-gen-worker,
# test/build/index_bg.wasm + test/build/index.js for worker) and snippets/
cd ts-gen-worker/test
node ../bench/coldstart.mjs /tmp/cs-tgw index.js /request 10
node ../bench/coldstart.mjs /tmp/cs-worker index.js /request 10
```

`/request` rather than `/` because the `worker` suite has no `GET /`. Section sizes
were read with a twenty-line Python section walker (the `(data ...)` segment identified
with `wasm-tools print`).

## Session 5: steady-state benchmark, worker vs ts-gen-worker vs plain JS

### Done

- `benchmark/js/index.js` (plain JS), `benchmark/src/lib.rs` (`worker`, extended; `/stream`
  and `/benchmark` kept) and `ts-gen-worker/benchmark/` (new workspace member, generated
  `Env` with `KV` and `SERVICE`), each serving `/hello /headers /echo /json /kv /stream
  /fanout` with each crate's idiomatic calls (`Response::ok` / `Response::builder().ok`,
  `req.json()` / `req.json_into()`, `kv.get(..).text()` / `kv.get(..)`, ...).
- `benchmark/build.sh` (`chomp build:bench`): one release pipeline for both Rust workers.
  `benchmark/run.js` (`chomp bench`): the driver. Suite unchanged: 194 passed, 4 skipped in
  both chomp tasks.

### Decisions

- **A fourth variant, `worker (no shim)`.** `worker` only runs under worker-build's entrypoint
  shim (a `WorkerEntrypoint` subclass behind `Proxy`s for trap recovery), which
  `ts-gen-worker` replaces with wasm-bindgen's own reinit. Without a control, any gap
  between the two would mix the shim with the bindings. The control is the same `.wasm`
  (byte-identical code; 23 bytes of producers metadata differ) and the same glue, run from a
  three-line `export default { fetch }` entry.
- **Identical pipeline.** Both Rust workers: workspace `[profile.release]`, the submodule's
  wasm-bindgen, then the same `wasm-opt -Oz` with the seven feature flags worker-build
  passes. `worker` is packaged by `worker-build --release --no-opt` with `NO_MINIFY=1`
  (its own `-O` pass replaced by the shared `-Oz`; unminified so the glue matches the
  control). `WASM_BINDGEN_USE_JS_SYS=1`, which worker-build sets, is not set for
  `ts-gen-worker`: it only switches the macro's paths from `wasm_bindgen_futures::` to
  `js_sys::futures::`, which the former re-exports.
- **Timing.** One Miniflare per variant, identical options (`compatibilityDate
  2026-03-17`, `KV` in-memory, `SERVICE` bound to the worker itself), all alive at once,
  requests interleaved one by one with the order rotated each round. `performance.now()`
  from `dispatchFetch` to the last body byte, response validated afterwards. Each repetition
  waits for a 1-minute load average below 2, measures cold start, starts fresh instances,
  sends 500 rounds of `/hello` to all of them (the first trial run showed Node's side of
  `dispatchFetch` still getting faster after 1000 requests: p50 1.65 → 0.73 ms), then per
  route discards 50 requests per variant and keeps 2000. req/s is sequential (one request in
  flight, `1000 / mean`), so it includes the harness.

### Results

`benchmark/run.js` defaults: 2000 requests per route per variant after 50 discarded, 3
repetitions, fresh instances each. Cells are the median over repetitions; "p50 range" is
the spread of the three p50s; "vs JS" is the median p50 minus plain JS's.

| route | variant | p50 ms | p50 range | p90 ms | p99 ms | req/s | vs JS µs |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `/hello` | js | 0.715 | 0.695–0.746 | 0.873 | 2.949 | 1266 | |
| | worker | 0.811 | 0.780–0.855 | 0.973 | 3.216 | 1115 | +96 |
| | worker (no shim) | 0.786 | 0.745–0.834 | 0.939 | 2.968 | 1161 | +71 |
| | ts-gen-worker | 0.774 | 0.741–0.784 | 0.925 | 3.240 | 1161 | +59 |
| `/headers` | js | 0.693 | 0.682–0.699 | 0.792 | 3.159 | 1293 | |
| | worker | 0.767 | 0.765–0.780 | 0.869 | 3.196 | 1195 | +74 |
| | worker (no shim) | 0.741 | 0.733–0.747 | 0.829 | 3.245 | 1228 | +48 |
| | ts-gen-worker | 0.736 | 0.731–0.750 | 0.828 | 3.244 | 1221 | +43 |
| `/echo` | js | 0.979 | 0.977–1.035 | 1.157 | 3.231 | 926 | |
| | worker | 1.121 | 1.117–1.143 | 1.345 | 3.456 | 793 | +142 |
| | worker (no shim) | 1.097 | 1.082–1.131 | 1.270 | 3.504 | 821 | +118 |
| | ts-gen-worker | 1.099 | 1.069–1.324 | 1.474 | 3.595 | 798 | +120 |
| `/json` | js | 0.912 | 0.908–0.914 | 1.000 | 2.995 | 1009 | |
| | worker | 1.042 | 1.041–1.052 | 1.153 | 3.256 | 885 | +130 |
| | worker (no shim) | 1.034 | 1.032–1.044 | 1.135 | 3.125 | 897 | +122 |
| | ts-gen-worker | 0.990 | 0.988–1.028 | 1.123 | 2.946 | 933 | +78 |
| `/kv` | js | 2.581 | 2.515–2.629 | 4.418 | 7.042 | 325 | |
| | worker | 2.734 | 2.682–2.763 | 4.464 | 7.197 | 306 | +153 |
| | worker (no shim) | 2.715 | 2.665–2.776 | 4.466 | 6.897 | 312 | +134 |
| | ts-gen-worker | 2.682 | 2.682–2.711 | 4.504 | 7.293 | 310 | +101 |
| `/stream` | js | 6.657 | 6.608–6.829 | 9.694 | 15.517 | 138 | |
| | worker | 7.189 | 7.178–7.253 | 10.206 | 24.235 | 127 | +532 |
| | worker (no shim) | 7.166 | 7.072–7.394 | 10.280 | 23.240 | 127 | +509 |
| | ts-gen-worker | 7.168 | 7.077–7.320 | 10.148 | 23.767 | 126 | +511 |
| `/fanout` | js | 1.094 | 1.088–1.095 | 1.278 | 3.902 | 772 | |
| | worker | 1.625 | 1.622–1.626 | 1.858 | 4.629 | 538 | +531 |
| | worker (no shim) | 1.555 | 1.553–1.556 | 1.791 | 4.438 | 556 | +461 |
| | ts-gen-worker | 1.275 | 1.274–1.285 | 1.525 | 4.189 | 675 | +181 |

Because the variants are interleaved, the paired difference within a repetition is tighter
than the ranges above. `ts-gen-worker` minus `worker (no shim)`, p50 µs, repetitions 1/2/3:
`/hello` −11/−50/−4, `/headers` 2/−5/−1, `/echo` −31/−27/+242, `/json` −46/−54/−5, `/kv`
−33/−64/+17, `/stream` 2/+248/−317, `/fanout` −270/−278/−283. The shim (`worker` minus
`worker (no shim)`): `/hello` 25/21/35, `/headers` 19/24/47, `/fanout` 67/73/68.

| | js | worker | worker (no shim) | ts-gen-worker |
| --- | ---: | ---: | ---: | ---: |
| `.wasm` after `-Oz` | | 398,223 | 398,200 | 120,933 |
| of which code / data | | 258,082 / 128,815 | same | 96,548 / 17,915 |
| `.wasm` after `-Oz` without the `/benchmark` route | | 210,978 | | |
| of which code / data | | 171,835 / 28,443 | | |
| JS shipped | 1,626 | 50,665 (shim + glue bundled) | 54,632 (entry + glue + snippet) | 33,991 |
| `grep -c '^function\|^const .* = function'` | | 26 | 21 | 23 |
| glue import shims (`__wbg_*: function`) | | 129 | 129 | 69 |
| wasm imports (`wasm-tools print \| grep -c '(import'`) | | 138 | 138 | 77 |
| wasm exports | | 54 | 54 | 33 |
| cold start ms, median of 10 (reps 1/2/3) | 127.5/112.2/125.3 | 139.8/125.3/134.2 | 140.9/124.0/131.8 | 130.5/120.6/126.5 |
| first request ms after `ready`, median of 10 | 3.75/4.90/3.09 | 6.24/8.24/5.36 | 6.63/8.22/5.35 | 5.48/7.29/4.50 |

Load average before/after each repetition: 1.51/2.13, 1.87/1.84, 1.84/1.56 (the benchmark's own
node and workerd processes account for most of it).

**Reading.** At p50, `ts-gen-worker` is not slower than `worker` on any route, and its cost over
plain JS on `/hello` through `/kv` is 43–120 µs per request where `worker`'s is 74–153 µs
(both about +510 µs on the 1 MB stream, where the shared `wasm_streams` path dominates). Two things move the comparison:
the shim, which costs `worker` about 20–35 µs per handler invocation, and `/fanout`, where 10
in-process sub-requests multiply the per-invocation cost without the Node transport: there
`ts-gen-worker` is 275 µs faster than `worker (no shim)`, about 25 µs per sub-request. Reading
the glue, `worker` does more per invocation (`Request::from` eagerly marshals `method`, `url`,
`headers` and `cf`; the response is a Rust-side builder converted to a `ResponseInit` with a
`Uint8Array` copy of the body; sub-request bodies are read with `arrayBuffer()`, copied and
UTF-8 validated), but which of these dominates was not isolated. Against the shim-free
control, the remaining differences on `/hello`, `/headers`, `/kv`, `/stream` are within the
paired noise; `/json` is 35 µs faster on average (the two serialize differently:
`serde_wasm_bindgen` into `Response.json()` versus a `serde_json` string; not isolated). The candidates the plan named are not visible as a cost: in this
wasm-bindgen neither glue has a `handleError` JS wrapper; `catch` imports are caught in wasm
via `WebAssembly.JSTag`, so the `--fallible` `Result`s cost nothing on the non-throwing path.
`JsNullable` adds no glue (`new Response(arg0, arg1)` directly). The visible extra
conversions in `ts-gen-worker` (`path()` parses the URL in JS: two more imports than
`worker`'s Rust-side scan; every `.header()` on the builder rebuilds the `Headers` from the
init, five times on `/headers`) are below the resolution of this harness. The one place
`ts-gen-worker` is slower is `/echo`'s tail (p90 +129 µs over `worker`, and one repetition's
p50 +242 µs over the control), which is the instance-level bimodality in Open problems. Sizes: `ts-gen-worker`'s `.wasm` is
0.57× `worker`'s once the legacy `/benchmark` route (which alone pulls in `url`/IDNA tables,
`Fetch` and `serde_json::json!`, 187 KB) is excluded, with about half the imports and glue
shims; cold start and first-request latency are a few ms lower, consistently across the three
repetitions but small against the 110–140 ms of process start.

### Open problems

- **`ts-gen-worker` `/echo` is bimodal per instance.** Across the main run and targeted
  `--routes echo` reruns, 13 repetitions were free of outside load (plain JS at its usual
  p50). In 3 of those 13 fresh `ts-gen-worker` instances (2 of 8 as benchmarked, one of them
  after 1000 warmup requests; 1 of 5 in the A/B below) the `/echo` p50 sat about 250 µs above
  its usual value for the whole window, and in 2 more for the first ~400 measured requests
  only. No `worker` or `worker (no shim)` instance did. 250 µs over 16 KB is about 15 ns per
  character, interpreter speed for a per-character loop. Swapping the response to
  `from_bytes` (the body form `worker` uses, removing `ts-gen-worker`'s only extra 16 KB step,
  the `TextDecoder` decode) did not remove it. The only per-character
  JS loop left on the path is wasm-bindgen's `passStringToWasm0` ASCII fast path in
  `text()`, identical in both glues; `--trace-opt` through `MINIFLARE_WORKERD_V8_FLAGS`
  shows it reaching Maglev and its TurboFan/OSR compile being requested in both variants,
  with completion rarely logged in either, so the traces did not separate them. Unexplained:
  why only `ts-gen-worker`'s instances land there. Proposal: a CPU profile of a slow instance
  through Miniflare's `inspectorPort` to see where the 250 µs goes; nothing is blocked on it.

### Commands

From the workspace root (needs `chomp build`: the submodule's wasm-bindgen and worker-build):

```sh
benchmark/build.sh            # or chomp build:bench
cd benchmark && node run.js   # or chomp bench; --out results.json keeps every sample
# targeted reruns used above
node run.js --routes echo --cold 0
node run.js --routes echo --cold 0 --warmup 1000
```

## Session 6: the benchmark on the restacked bindings

### Done

- `gbedford/tgw-crate` reset to `ceb6811` (bindings from ts-gen `2171ea7`, wasm-bindgen
  `1ea319f65`), with the two Session 5 benchmark commits re-applied on top since `ceb6811`
  does not carry them. `ts-gen` reinstalled from the submodule; `build:ts-gen-worker:types`
  leaves `ts-gen-worker/benchmark/src/env.rs` unchanged.
- Session 5 protocol rerun unchanged: `chomp build:bench`, `node run.js` (3 repetitions × 2000
  per route per variant after 50 discarded, all four variants interleaved, load before each
  repetition 1.79 / 1.75 / 1.91).
- Migration table and `MIGRATING.md` updated for the `parity/workers-types.md` deltas (see
  Decisions).

### Results

The `worker` and `worker (no shim)` builds are byte-identical to Session 5 (`.wasm` and glue),
so they measure machine drift only. Δ is Session 6 minus Session 5 of the median p50; the
paired column is `ts-gen-worker` minus `worker` p50 within each repetition (µs, reps 1/2/3),
which cancels drift that hits both.

| route | ts-gen-worker S5 → S6 ms | Δ µs | worker S5 → S6 ms | Δ µs | js Δ µs | paired S5 | paired S6 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `/hello` | 0.774 → 0.759 | −15 | 0.811 → 0.790 | −21 | −4 | −36/−71/−39 | −56/−28/−22 |
| `/headers` | 0.736 → 0.742 | +6 | 0.767 → 0.762 | −5 | +5 | −17/−28/−48 | −40/−20/−29 |
| `/echo` | 1.099 → 1.079 | −20 | 1.121 → 1.119 | −1 | +8 | −44/−51/+207 | −32/−47/−29 |
| `/json` | 0.990 → 0.998 | +7 | 1.042 → 1.062 | +21 | +21 | −53/−51/−25 | −62/−66/−63 |
| `/kv` | 2.682 → 2.501 | −181 | 2.734 → 2.544 | −190 | −177 | −81/−23/−1 | −57/−52/−36 |
| `/stream` | 7.168 → 7.354 | +186 | 7.189 → 7.459 | +271 | +437 | −10/+67/−111 | +15/−93/−105 |
| `/fanout` | 1.275 → 1.286 | +11 | 1.625 → 1.660 | +35 | +15 | −338/−351/−351 | −374/−370/−331 |

| ts-gen-worker | Session 5 | Session 6 |
| --- | ---: | ---: |
| `.wasm` after `-Oz` | 120,933 | 120,656 |
| of which code / data | 96,548 / 17,915 | 96,318 / 17,915 |
| glue `index.js` | 33,991 | 33,871 |
| `grep -c '^function\|^const .* = function'` | 23 | 23 |
| glue import shims / wasm imports / wasm exports | 69 / 77 / 33 | 69 / 77 / 31 |
| cold start ms, median of 10 (reps 1/2/3) | 130.5/120.6/126.5 | 119.0/123.8/138.2 |
| first request ms after `ready` (reps 1/2/3) | 5.48/7.29/4.50 | 7.19/5.52/4.54 |

`worker` cold start in Session 6: 124.3/123.1/140.3 ms, first request 8.71/8.51/5.44 ms.

**Reading.** No route got slower. The absolute p50s moved by the same amount for all four
variants, plain JS included (`/kv` about −180 µs, `/stream` +186 to +437 µs), and `worker`,
whose code did not change, moved further than `ts-gen-worker` on five of seven routes
(`/hello`, `/json`, `/kv`, `/stream`, `/fanout`); those deltas are machine drift. Paired against
`worker`, the median moves by −83 to +12 µs per route; the two positive ones (`/hello` +11,
`/echo` +12) are inside the Session 6 spread of the paired values (34 and 18 µs). For scale,
the shim's cost (`worker` minus `worker (no shim)`, the same bytes in both sessions) went from
21–35 to −3–18 µs on `/hello` and from 19–47 to 1–10 µs on `/headers`: cross-session
differences of about 25 µs are not attributable to anything. That matches the glue diff, which
touches no route's imports: the variadic `console.error` reads its arguments as a view instead
of copying and freeing them (drops the `__externref_drop_slice` and `__wbindgen_free` exports;
reached only from the error path), the `CONSOLE` accessor is renamed `console`, and the
unused `RotateKind` is no longer exported. The import list is otherwise identical.

### Decisions

- **Both workers-types delta lists applied.** Session 11's list (lower-case globals, enum
  options, `Vec` returns, `WebSocket` not `Eq`, the `cloudflare::*` paths) and the earlier
  rebased-stack list (`JsNullable` arguments), since the table still showed the pre-#51 forms
  (`.web_socket(&ws)`, `.body_with_str(..)`). Rows changed: queue `messages()`, email
  `EmailMessage` path, `.with_websocket`, `.with_encode_body`, `RequestInit` body, KV list
  `keys()`; rows added: `.with_cache`, `Cache::default()` (with the globals), `WebSocket: Eq`.
  The R2, `KVNamespaceListOptions`, `clear_timeout` and `EmailAttachment` deltas have no row:
  the table excludes those APIs, or `ext/` hides the change (the KV list builder). Every new
  form is checked against `bindings.rs` and the port's call sites.

### Open problems

- **The `/echo` slow mode is in the shim-free entry, not in the bindings** (update to
  Session 5's open problem). This run, `worker (no shim)` was in it for all of repetition 1
  (p50 1.488 ms, +285 µs over `worker`) and for the first 400 measured requests of
  repetition 2. `ts-gen-worker` was in it for the first 400 requests of every repetition
  (median of the first 400 +276 to +382 µs above the last 1600), which is where its `/echo`
  p90 (1.502 vs `worker` 1.296 ms) comes from. `worker` with its shim never was, in either
  session (its first 400 sit at most +122 µs above the rest, as plain JS's do). The two
  variants that show it share the direct `export default` entry (no `WorkerEntrypoint` class
  or `Proxy` in front of the export) and differ in glue, so the generated bindings are not
  needed to produce it. The CPU-profile proposal stands,
  now comparing `worker` to `worker (no shim)`; a longer per-route warmup would take the
  effect out of the steady-state numbers.

### Commands

As Session 5, plus the comparison scripts (not committed):

```sh
benchmark/build.sh
cd benchmark && node run.js --out /tmp/opencode/wr4-s6-results.json
node /tmp/opencode/wr4-s6-analyze.mjs /tmp/opencode/bench-session5.json /tmp/opencode/wr4-s6-results.json
node /tmp/opencode/wr4-s6-echo.mjs /tmp/opencode/wr4-s6-results.json echo
```

## Migration table

The non-binding `worker` surface used in `test/src/*.rs` (plus the handler kinds and context
calls it does not use), and its `ts-gen-worker` form, as the code is after Session 6. Rows from
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
| `#[event(queue)] async fn q(batch: MessageBatch<T>, ..) -> Result<()>` | `batch: MessageBatch`; `batch.messages()` is a `Vec<Message>` (no `?`); `message.body_into::<T>()?` |
| `#[event(scheduled)] async fn s(event: ScheduledEvent, env, ctx: ScheduleContext)` | `(controller: ScheduledController, env, ctx: ExecutionContext)` |
| `#[event(email)]` | same (`EmailMessage` re-exported at the root; canonical `cloudflare::email::EmailMessage`) |
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
| `.with_websocket(ws)` | `.web_socket(&JsNullable::wrap(ws))` (or the terminal `.from_websocket(ws)`) |
| `.with_encode_body(EncodeBody::Manual)` | `.encode_body(EncodeBodyKind::Manual)` |
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
| `RequestInit::new().with_method(..)` / `.with_body(Some(v))` / `.with_headers(h)` | `RequestInit::builder().method(..)` / `.body_with_str(&JsNullable::wrap(s.into()))` / `.headers(&h)`, then `.build()` |
| `.with_cache(CacheMode::NoStore)` | `.cache(CacheKind::NoStore)` |
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
| `HeaderMap` ↔ `Headers` (feature `http`) | `TryFrom` both ways (non-ASCII value, DEL byte are errors) |
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
| `list.keys[i].name` / `.expiration: Option<u64>` / `list.list_complete` / `list.cursor` | `list.keys()[i].name()` (`Vec`) / `.expiration() -> Option<f64>` / `list.list_complete()` / `list.cursor()` |
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
| `Cache::default()` | `caches.with(CacheStorage::default)` (JS globals are lower-case statics: `console`, `caches`, `crypto`, `performance`) |
| `WebSocket: Eq` (`HashSet<WebSocket>`) | `PartialEq` only (ts-gen bug 2 in `parity/workers-types.md`) |
