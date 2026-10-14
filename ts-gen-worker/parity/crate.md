# crate: `#[event]` macros and the `impl` layer

Owner: workers-rs-4. Branch `gbedford/tgw-crate` (on `gbedford/ts-gen-worker` with the error
model and the test-suite port); ts-gen branch `agent/crate` (one commit, on
`gbedford/workers-parity`).

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
- `src/ext/streams.rs`: `ByteStream`, `Request::bytes_stream()`, `Response::bytes_stream()`,
  `ResponseInitBuilder::from_stream(S)`, `readable_stream_from(S)`.
- `src/ext/response.rs`: `Response::builder()` → the generated `ResponseInitBuilder`, carrying
  `ok`, `error`, `empty`, `from_html`, `from_bytes` (+ `from_json`, `from_stream` above).
- `src/router.rs`: `Router<'a, E, D = ()>` / `RouteContext<E, D>` over the generated types.
- `src/durable.rs`: `DurableObject` trait with `type Env`, `fetch`, `alarm`.
- ts-gen `a643100` (agent/crate): type-parameter defaults land on the `pub type`
  declaration (`MessageBatch<Body: JsGeneric = JsValue>`), so a defaulted generic is a
  complete bare name, as is the constructor return of a class whose parameters all default.
  Fixture, integration compile test, CONVENTIONS section. No output change for this crate
  today: the port's declarations drop generics; the real workers-types input
  (`ExecutionContext<Props = unknown>`, `MessageBatch<Body = unknown>`) needs it.
- Tests, append-only in the ported tree: `test/src/ext.rs` (routes under `/ext/*`, a
  `MacroCounter` `#[durable_object]`, `scheduled`/`email` handlers, a second `start`), one
  arm in `lib.rs`'s dispatch, one `env.d.ts` line, and `test/tests/ext.spec.ts` with its own
  Miniflare instance (`unsafeTriggerHandlers`): 15 cases, one per feature, including both
  fetch error modes via `__macro::error_response`. Suite: 176 passed, 6 skipped (161 before).

## Port adoption

The ported test worker hand-rolls what this layer now provides. Each is a follow-up, not
done in the integration step; the port's test-local `util.rs` is the concrete list.

- `fetch_handler` (`#[wasm_bindgen(js_name = "fetch")]` + `error(err_message(&e), 500)`) →
  `#[event(fetch, respond_with_errors)]`. Difference to settle first: the port renders a caught
  JS error as `Name: message` (`util::err_message`, `worker::Error`'s format, asserted by
  `sqlite.spec.ts`'s `"Error: Cannot store value"`); `respond_with_errors` renders
  `Display` of `Error`, which is the message alone. Either `Error`'s `Display` adopts the
  `Name: message` form for `Js` (then `worker.spec.ts`'s `error-display` case changes) or
  the macro does.
- `start` + `second_start` (one `#[wasm_bindgen(start)]` calling both, with a GAP comment)
  → two `#[event(start)]`. wasm-bindgen does accept several `start` functions;
  `ext.spec.ts` runs three.
- `queue` export (`#[wasm_bindgen(js_name = "queue")]`, `serde_wasm_bindgen::from_value(message.body())`)
  → `#[event(queue)]` and `message.body_into::<QueueBody>()?`.
- DO classes (`Counter`, `PutRawTestObject`, `AutoResponseObject`, `MyClass`, `SqlCounter`,
  `SqlIterator`, `EchoContainer`: `#[wasm_bindgen]` struct + constructor + `fetch`) →
  `#[durable_object]` + `impl DurableObject`. Blocked for the websocket ones until the trait
  grows `websocket_message/close/error` (the port's `webSocket*` methods show the shapes;
  `WebSocket`/`MessageEvent`/`CloseEvent` are declared now).
- `route()`'s `(method, segments)` slice match → `Router<Env, SomeSharedData>` with
  `or_else_any_method_async` for the catchall. Worth doing only if the per-route handler
  signatures (`(req, env, data)`) move to `(req, RouteContext)`; otherwise the slice match is
  the cheaper dispatch.
- `util::ok/error/from_json/from_bytes` → `Response::builder().ok/error/from_json/from_bytes`;
  `util::from_websocket` → `Response::builder().status(101.0).web_socket(&ws).empty()`;
  `util::url/path` → `req.url_parsed()?` (feature `url`) / `req.path()`.
- `util::sleep` (`setTimeout` + `Promise`) → a crate `Delay` future (gap below); until then
  the port keeps it.
- `util::storage_get/storage_put` (serde over `DurableObjectStorage`), `util::block_concurrency_while`
  (Rust future inside the input gate), `util::websocket_events`/`on_websocket_events`
  (`WebSocket::events()` stream), `util::nullable` (`JsOption` `null`) are crate-layer
  candidates, listed under gaps.
- `connect` export → `#[event(connect)]`, not ported.

## Decisions

- **Handler wrapper placement.** `#[event]` leaves the user function untouched and emits the
  wasm-bindgen export inside `const _: () = { ... }` next to it, with the parameter types
  copied verbatim. Block scope sees the module's items and imports, so the app's generated
  `Env` resolves without the macro knowing its path, and nothing new enters the module
  namespace. Unlike `worker-macros` there is no renaming of the user fn.
- **Error bound is `Into<JsValue>` only**, in both modes. The plan said
  `Display + Into<JsValue>` for `respond_with_errors`; instead the macro converts the error to
  `JsValue` and takes the body text from `Error::from(v).to_string()`, i.e. `JsException`'s
  `Display`. One bound, and handlers returning `Result<_, JsValue>` or `Result<_, JsException>`
  straight from the bindings work in both modes. The error is logged via `console.error` as
  the JS value, so JS errors keep their stacks in logs. `respond_with_errors` on any other
  kind is a compile error (`worker` ignored it).
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
  `worker`'s `Result<Response>`. WebSocket handlers are not in the trait yet (Port adoption).
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

- **Feature `http` means something else than in `worker`.** There it switches the `fetch`
  handler to `http::Request`/`http::Response` (`test/src/lib.rs` builds both ways through
  `HandlerRequest`); here it only adds `HeaderMap` conversions, so a `worker` user enabling
  it gets no error and no `http` handler. Proposal: port the `Request`/`Response` ↔
  `http::Request`/`http::Response` conversions under the same feature and have `#[event(fetch)]`
  convert through a trait when the signature names `http` types. Blocked: the `http` half of
  the test suite.
- **No guard that a `DurableObject` impl is exported.** `worker` makes `DurableObject` require
  a `has_durable_object_attribute` marker that only `#[durable_object]` implements; here an
  impl without the attribute compiles and exports nothing. Proposal: the same hidden marker
  supertrait, emitted by the macro.
- **`Error` `Display` format** decides whether `respond_with_errors` can replace the port's
  `fetch_handler` (Port adoption, first item). Proposal: `Js` displays `Name: message` for an
  `Error` instance, as `worker::Error` did, since migrating specs assert on it.
- **`JsException` `Display` for a thrown primitive** falls through to `Debug`
  (`JsValue(42)`, `JsValue(null)`); `String(v)` would give `42`/`null`. Minor; surfaces in
  `respond_with_errors` bodies. For the errors owner.

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
- ts-gen: `number` lowers to `f64` everywhere, so `status(201.0)`, `status() -> f64`,
  `raw_size() -> f64`. `worker` used `u16`/`u64`. Needs an integer heuristic or annotation.
- ts-gen: `Promise<string>` returns `JsString`; every `text().await?` needs `String::from`.
- Env generation: an app `env.d.ts` referencing crate types needs
  `--external "X=::ts_gen_worker::X"` per type (eleven in the chompfile now). A fallback
  mapping for unresolved globals to a crate path would remove that.
- Crate-layer idioms the port had to write locally (`test/src/util.rs`): `console_log!`
  and friends, `Delay`, serde `get`/`put` over `DurableObjectStorage`,
  `block_concurrency_while` over a Rust future, a `WebSocket::events()` stream, DO
  websocket handlers in the `DurableObject` trait, `#[event(connect)]`, `Response::from_body`/
  `into_parts`, the `http` feature's handler types.

## Migration table

The non-binding `worker` surface used in `test/src/*.rs` (plus the handler kinds and context
calls it does not use), and its `ts-gen-worker` form. Binding APIs (KV, R2, D1, Cache, queue
producers, DO storage beyond `get`/`put`, SQL storage, sockets, rate limiting, analytics,
secrets store, containers, assets, `send_email`) are not listed: `parity/test-port.md` has
them module by module. Under `--fallible` every sync binding that can throw returns `Result`,
so `worker`'s `?` stays where it was and appears on a few calls that had none.

| `worker` | `ts-gen-worker` |
| --- | --- |
| `#[event(fetch)] async fn f(req: Request, env: Env, ctx: Context)` | same, with `ctx: ExecutionContext` and `env` the app's type generated from its `env.d.ts` |
| `#[event(fetch, respond_with_errors)]` | same |
| `#[event(fetch)]` over `http::Request` (feature `http`) | not ported |
| `#[event(start)]` (several allowed) | same |
| `#[event(queue)] async fn q(batch: MessageBatch<T>, ..) -> Result<()>` | `batch: MessageBatch`; `message.body_into::<T>()?` |
| `#[event(scheduled)] async fn s(event: ScheduledEvent, env, ctx: ScheduleContext)` | `(controller: ScheduledController, env, ctx: ExecutionContext)` |
| `#[event(email)]` | same |
| `#[event(connect)]` | not ported |
| `#[durable_object] struct S; impl DurableObject for S { fn new(state: State, env: Env) }` | same, plus `type Env = Env;`, `state: DurableObjectState` |
| `async fn alarm(&self) -> Result<Response>` | `-> Result<()>` |
| `Response::ok(body)` | `Response::builder().ok(body)` |
| `Response::error(msg, status)` | `Response::builder().error(msg, status)` |
| `Response::from_json(&v)` | `Response::builder().from_json(&v)` |
| `Response::from_bytes(v)` | `Response::builder().from_bytes(v)` |
| `Response::from_html(s)` | `Response::builder().from_html(s)` |
| `Response::from_stream(s)` | `Response::builder().from_stream(s)` |
| `Response::empty()` | `Response::builder().empty()` |
| `Response::builder()` / `ResponseBuilder::new()` | `Response::builder()` (the generated `ResponseInitBuilder`) |
| `.with_status(201)` | `.status(201.0)` on the builder, before the terminal call |
| `.with_headers(h)` | `.headers(&h)` on the builder |
| `.with_header(k, v)?` | `.headers(&[(k, v)].into_iter().collect())` on the builder |
| `builder.body(body)` after `res.into_parts()` | not ported |
| `.with_websocket(ws)` / `.with_cf(..)` / `.with_encode_body(..)` / `.with_cors(..)` | not yet: `webSocket`/`cf`/`encodeBody` arrive with the real `ResponseInit`; no `Cors` |
| `Response::redirect(url)` / `redirect_with_status(url, 302)` | `Response::redirect(url.as_str())?` / `Response::redirect_with_status(url.as_str(), 302.0)?` |
| `Response::from_websocket(ws)` / `res.websocket()` | `Response::builder().status(101.0).web_socket(&ws).empty()` / `res.web_socket()` |
| `res.status_code()` | `res.status() as u16` |
| `res.text().await?` | `String::from(res.text().await?)` |
| `res.json::<T>().await?` | `res.json_into::<T>().await?` |
| `res.bytes().await?` | `res.bytes().await?.to_vec()` |
| `res.stream()?` | `res.bytes_stream()` (`Option`) |
| `res.cloned()?` | `res.clone()?` |
| `res.headers()` / `res.headers_mut()` | `res.headers()` (JS `Headers` is mutable) |
| `Request::new(url, Method::Post)` | `Request::new_with_str_and_init(url, &RequestInit::builder().method("POST").build())` |
| `Request::new_with_init(url, &init)` | `Request::new_with_str_and_init(url, &init)` |
| `RequestInit::new().with_method(..)` / `.with_body(Some(v))` | `RequestInit::builder().method(..)` / `.body(..)`, then `.build()` |
| `Fetch::Url(url).send().await?` | `fetch_with_str(url.as_str()).await?` |
| `Fetch::Request(req).send().await?` | `fetch(&req).await?` |
| `fetch.send_with_signal(&signal)` / `AbortController::default()` | `fetch_with_str_and_init(url, &RequestInit::builder().signal(&c.signal()).build())` / `AbortController::new()?` |
| `req.method()` → `Method` | `req.method()` → `String`; `req.method().parse::<Method>()?` |
| `req.path()` | same |
| `req.url()?` → `url::Url` | `req.url()` → `String`; `req.url_parsed()?` (feature `url`) |
| `req.query::<Q>()?` | same |
| `req.text().await?` | `String::from(req.text().await?)` |
| `req.json::<T>().await?` | `req.json_into::<T>().await?` |
| `req.bytes().await?` | `req.bytes().await?.to_vec()` |
| `req.stream()?` | `req.bytes_stream()` |
| `req.form_data().await?` | same (`FormData.get` is a `JsValue`) |
| `req.headers()` / `req.headers_mut()?` | `req.headers()` |
| `req.cf()` | `req.cf()` (`Option<IncomingRequestCfProperties>`) |
| `req.clone()?` | same |
| `Headers::new()` | `Headers::new()?` or `Headers::default()` |
| `headers.get(k)?` / `has(k)?` / `set(k, v)?` / `append(k, v)?` / `delete(k)?` | same (`JsException::TypeError` on a bad name) |
| `headers.get_all(k)?` | same (`Set-Cookie` only under workerd) |
| `headers.entries()` / `for (k, v) in &headers` | same |
| `headers.keys()` / `values()` | generated `js_sys::Iterator<JsString>` |
| `ctx.wait_until(fut)` | `ctx.spawn(fut)?` |
| `ctx.pass_through_on_exception()` | `ctx.pass_through_on_exception()?` |
| `Router::new()` / `Router::with_data(d)` | `Router::<Env>::new()` / `Router::<Env, D>::with_data(d)` |
| `.get/post/.._async(pattern, handler)` / `.run(req, env)` | same |
| `ctx.param("id")` / `ctx.data` / `ctx.env` | same |
| `ctx.kv("X")?` / `ctx.secret("X")?` / `ctx.durable_object("X")?` | `ctx.env.x()` (generated getters) |
| `env.kv(..)` / `bucket` / `d1` / `queue` / `service` / `secret` / `var` / `durable_object` | `env.<binding>()` generated from the app's `env.d.ts` |
| `env.durable_object("X")?.id_from_name(n)?.get_stub()?` | `env.x().get(&env.x().id_from_name(n)?)?` or `env.x().get_by_name(n)?` |
| `stub.fetch_with_str(url).await?` | same |
| `stub.fetch_with_request(req).await?` | `stub.fetch(&req).await?` |
| `state.storage().get::<T>(k).await?` | `storage.get(k).await?` → `JsValue` (`undefined` when absent; serde form is a gap) |
| `state.storage().put(k, v).await?` | `storage.put(k, &js_value).await?` |
| `Error::from(..)` / `Error::RustError(s)` / `Error::D1(e)` | `Error::from(s)` / `Error::Rust(s)` / `Error::Js(JsException::..)` |
| `console_log!(..)` | `console::log(&[format!(..).into()])` (no macro yet) |
| `Date::now().as_millis()` | `js_sys::Date::now() as u64` |
| `Date::new(DateInit::Millis(n))` / `DateInit::String(s)` | `js_sys::Date::new(&(n as f64).into())` / `js_sys::Date::new(&s.into())` |
| `Delay::from(duration).await` | not ported (`setTimeout` is declared; the port's `util::sleep`) |
