# Migrating from `worker` to `ts-gen-worker`

The bindings are generated from `@cloudflare/workers-types`, so the JS names are the
Rust names: `webSocket` is `web_socket()`, a `ResponseInit` is built with the generated
`ResponseInit::builder()`, and the overloads of a JS method are `new_with_str`,
`new_with_str_and_init`, and so on. What `worker` added on top of the platform (the
`#[event]` macros, `Result`, serde, `Method`, `Router`, `ByteStream`, Durable Object
classes) is still there as `impl` blocks on the generated types with the same shapes
wherever a name was free. This guide covers the changes a typical worker meets.

Every sync binding that can throw returns `Result`, so a few calls gain a `?`
(`Headers::new()?`, `res.clone()?`). Async methods and Promise-typed getters (`writer.ready()`,
`socket.opened()`) return a typed `Promise<T, Error>` that awaits to `Result<T, Error>`
(`text().await?` is a `String`) and passes back to JS (`p.upcast()` where a `&Promise` is taken).

The bindings are generated with mono generics on js-sys's unstable API and wasm-bindgen's generic
sequences, so an app builds with `--cfg js_sys_unstable_apis`
(in its `.cargo/config.toml` rustflags or `RUSTFLAGS`), and js-sys
collections behave like Rust collections: `map.get(&"k".into())`, `for (k, v) in &headers`, `record.iter()`,
`promise.await`. A string inside a collection is a `JsString` handle (`String::from(k)`): `&headers`
yields `(JsString, JsString)`, a KV bulk read is a `Map<JsString, …>`.

String arguments take `impl JsStringLike` (`&str`, `String`, `&String`), and string lists take
`&[impl JsStringLike]` (`&["a", "b"]`). A list of a union is a `&[JsValue]`
(`.indexes(&["a".into()])`, `Blob::new_with_bits(&[…])`). A nullable argument, setter or builder value takes the
value bare and sends `null` through its own `_with_null` form (`clear_timeout(id)` /
`clear_timeout_with_null()`, `.prefix(p)` / `.prefix_with_null()`); a nullable argument that is also
overloaded takes `Option<…>` (`Response::new_with_readable_stream_and_init(None, &init)`). An optional one is an overload or builder method left uncalled; only an
argument or setter whose type admits `undefined` takes `Option<…>`. Reads fold `null` and `undefined` into `None`; an `any`
read stays a `JsValue` (`ws.deserialize_attachment()?`, `storage.get(k)`), checked with `is_undefined()` / `is_null()`. Types that are only ever read
(`KVNamespaceListResult`, `R2Objects`, Queue metrics) have getters only: no `new`, setters or
builder. So do the properties workerd makes read-only (`res.status()`, `req.url()`,
`ws.ready_state()`): there is no `set_status`. Byte arguments take a buffer handle, one overload
per kind: `&ArrayBuffer` (`bucket.put_with_array_buffer(k, &ArrayBuffer::from(&bytes[..]))`,
`.md5(&ArrayBuffer::from(&digest[..]))`) or a view (`put_with_typed_array(k, &Uint8Array::from(&bytes[..]))`);
`Response::builder().from_bytes(&bytes)` and `kv.put_bytes(k, &bytes)` take the `&[u8]`. Callbacks are `Function::from_fn(|event: MessageEvent| …)`, or `Function::once(|| …)` for
a one-shot; an event listener returns `Undefined::UNDEFINED`. String-enum
options take the generated enum (`ResponseInitEncodeBody::Manual`, `RequestInitCache::NoStore`,
`WebSocketBinaryType::Arraybuffer`), and arrays come back as `Vec<T>`. The JS globals are
lower-case statics: `console`, `caches`, `crypto`, `performance` (`caches.with(CacheStorage::default)`
is `worker`'s `Cache::default()`). `PerformanceMark` has no constructor here;
`performance.with(|p| p.mark(name))?` creates one.

## Env

`worker::Env` was dynamic (`env.kv("X")?`). Here the app declares its bindings in a
`.d.ts` and generates a typed `Env`:

```ts
// env.d.ts
interface Env {
  GREETING: string;
  TEST: KVNamespace;
  COUNTER: DurableObjectNamespace;
}
```

```rust
mod env; // ts-gen output

let kv = env.test();           // KVNamespace
let greeting = env.greeting(); // String
```

The dynamic forms stay for code that needs them (`EnvExt`): `env.secret("X")?`,
`env.var("X")?`, `env.object_var::<T>("X")?`, `env.get::<KVNamespace>("X")?`.

## Handlers

```rust
// worker
#[event(fetch)]
async fn main(req: Request, env: Env, ctx: Context) -> Result<Response> { .. }

// ts-gen-worker
#[event(fetch)]
async fn main(req: Request, env: Env, ctx: ExecutionContext) -> Result<Response> { .. }
```

`respond_with_errors`, `start`, `queue`, `email`, `connect` and the `http` feature's
`http::Request`/`http::Response` handler signature work as before. Differences:

* `scheduled` receives a `ScheduledController` and an `ExecutionContext`.
* `queue` receives an untyped `MessageBatch`; `batch.messages()` is a `Vec<Message>`;
  deserialize each message with `message.body_into::<T>()?`.
* `connect` has no `(udp)` form: one handler serves TCP and UDP and dispatches on the
  socket's protocol.
* `ctx.wait_until(fut)` is `ctx.spawn(fut)`; the generated `wait_until(&Promise)` keeps
  its JS shape.

## Request and Response

Constructors move to the builder, which is the generated `ResponseInit` builder plus
`worker`'s terminal shortcuts:

```rust
// worker
Response::ok("hi")
Response::error("nope", 404)
Response::from_json(&v)
Response::from_websocket(ws)
ResponseBuilder::new().with_status(201).with_header("x-a", "1")?.ok("hi")

// ts-gen-worker
Response::builder().ok("hi")
Response::builder().error("nope", 404)
Response::builder().from_json(&v)
Response::builder().from_websocket(ws)
Response::builder().status(201).header("x-a", "1")?.ok("hi")
```

`with_headers(h)` is `.headers(&h)`, `with_cors(&c)?` is `.cors(&c)?`, `with_websocket(ws)`
is `.web_socket(&ws)`, `with_encode_body(EncodeBody::Manual)` is
`.encode_body(ResponseInitEncodeBody::Manual)`, `with_cf(v)?` is
`.cf(&serde_wasm_bindgen::to_value(&v)?)`.

A JS `Response` is immutable, so `res.with_status(..)` does not exist; split and
re-construct as `worker`'s `into_parts` did:

```rust
let (builder, body) = upstream.into_parts();
builder.status(404).body(body)?
```

Reading:

```rust
// worker                       // ts-gen-worker
res.status_code()               res.status()
res.json::<T>().await?          res.json_into::<T>().await?
res.bytes().await?              res.bytes().await?.to_vec()
res.stream()?                   res.bytes_stream()   // Option<ByteStream>
res.cloned()?                   res.clone()?
res.websocket()                 res.web_socket()
req.method()                    req.method().parse::<Method>()?   // method() is a String
req.url()?                      req.url_parsed()?    // feature `url`; url() is a String
req.json::<T>().await?          req.json_into::<T>().await?
```

`req.path()`, `req.query::<Q>()?`, `req.headers()`, `req.cf()`, `req.clone()?`,
`req.text().await?`, `req.form_data().await?` are unchanged. `headers_mut()` is gone: a JS
`Headers` is already mutable through `headers()`.

Building a request:

```rust
// worker
Request::new(url, Method::Post)?
Request::new_with_init(url, &RequestInit::new().with_method(Method::Post).with_body(Some(b)))?

// ts-gen-worker
Request::new_with_method(url, Method::Post)?
Request::new_with_str_and_init(
    url,
    &RequestInit::builder().method("POST").body_with_str(b).build(),
)?
```

`RequestInit` is generic over its `cf` type, named at the builder:
`fetch_with_str_and_init` takes a `RequestInit<RequestInitCfProperties>`, built with
`RequestInit::<RequestInitCfProperties>::builder()`, and `Request::new_with_str_and_init`
returns a `Request<CfHostMetadata, Cf>` for the init's `Cf`
(`Request::<JsValue, _>::new_with_str_and_init(url, &init)` where nothing else fixes it).

`with_cache(CacheMode::NoStore)` is `.cache(RequestInitCache::NoStore)`.

`Fetch::Url(..).send()`, `Fetch::Request(..).send_with_signal(&signal)` and
`AbortController::default()` are unchanged (`Fetch::Url` holds a `String`; the generated
`AbortController::new()` returns a `Result`); the generated
`fetch_with_str(url)` / `fetch(&req)` are the direct forms.

## Headers

`Headers::new()?` or `Headers::default()`; `get`/`has`/`set`/`append`/`delete`/`get_all`
as before; `for (k, v) in &headers` and `[(k, v)].into_iter().collect::<Headers>()` as
before. `keys()`/`values()` iterate as `String`. `HeaderMap` converts both
ways under the `http` feature (`try_into()`: a value JS cannot hold is an error).

## Durable Objects

```rust
#[durable_object]
struct Counter { state: DurableObjectState, env: Env }

impl DurableObject for Counter {
    type Env = Env;
    fn new(state: DurableObjectState, env: Env) -> Self { Self { state, env } }
    async fn fetch(&self, req: Request) -> Result<Response> { .. }
    async fn alarm(&self) -> Result<()> { .. }   // was Result<Response>
}
```

`websocket_message` receives a `DataKind` (`String(s)` / `ArrayBuffer(buf)`) instead of
`WebSocketIncomingMessage`; `websocket_close`/`websocket_error` are unchanged. `WebSocket`
is `PartialEq` and `Eq` by handle, as in `worker`, so a struct deriving `Eq` can hold one.
`worker`'s `ws.events()` stream is a listener per event type:
`ws.add_event_listener("message", f.unchecked_ref())?` with
`f: Function<fn(MessageEvent) -> Undefined> = Function::from_fn(|e: MessageEvent| …)`,
and the same for `"close"` and `"error"`. `e.data()` is a `JsValue`: a string, an `ArrayBuffer`, or a `Blob` under the
`websocket_standard_binary_type` compat flag (the default from 2026-03-17) unless `binaryType` is set,
or anything else (a `MessagePort` message is any structured clone).
`FormData::get` returns `Option<JsValue>` (a `File` by `dyn_into::<File>()`, else a string by
`as_string()`), and `get_all` a `Vec<JsValue>`.

```rust
// worker                                         // ts-gen-worker
env.durable_object("X")?.id_from_name(n)?.get_stub()?   env.x().get_by_name(n)?
stub.fetch_with_request(req).await?               stub.fetch(&req).await?
state.storage().get::<T>(k).await?                storage.get_into::<T>(k).await?
state.storage().get_with_options::<T>(k, o).await?  storage.get_into_with_options::<T>(k, &o).await?
state.storage().get_multiple(ks).await?           storage.get_with_keys(&ks).await?
state.storage().get_multiple_with_options(ks, o)  storage.get_with_keys_and_options(&ks, &o)
state.storage().put(k, v).await?                  storage.put_serialized(k, &v).await?
```

The storage option structs (`DurableObjectGetOptions`, `GetAlarmOptions`, `SetAlarmOptions`,
`ListOptions`) become the generated `DurableObject*Options` types, built with
`DurableObjectGetOptions::builder().no_cache(true).build()` and passed by reference.

The generated `storage.get::<T>(k)` returns `Option<T>` for a number, string or handle
(`storage.get::<f64>("count")`; `storage.get::<JsValue>(k)` is the untyped value), and `storage.put(k, v)`
takes the value: `f64`, `String` or a handle.
`storage.put_entries_serialized(&s)` stores a struct's fields as entries (`put_multiple`).

## KV

Plain reads and writes are shorter: `kv.put("k", "v").await?`, `kv.get("k").await?`.
The fluent forms carry a `_builder` suffix because the generated `get`/`put`/`list` own
the JS names:

```rust
// worker                                   // ts-gen-worker
kv.get("k").json::<T>().await?              kv.get_builder("k").json::<T>().await?
kv.put("k", &t)?.expiration_ttl(60).execute().await?
                                            kv.put_builder("k", &t)?.expiration_ttl(60).execute().await?
kv.list().prefix(p).execute().await?        kv.list_builder().prefix(p).execute().await?
```

`get_bulk`, `put_bytes`, `put_stream` keep their names. The generated reads underneath are
typed per value type without options (`kv.get_text(k)`, `get_array_buffer(k)`, `get_stream(k)`,
`get_with_metadata_text(k)`, whose `metadata()` is a `JsValue`); with options only the text read
is generated (`kv.get_with_str_and_kv_namespace_get_options(k, &opts)`, `opts` built by
`KVNamespaceGetOptions::builder("text".to_owned())`), so `get_builder` carries the `cache_ttl`
forms of the others. The bulk read is `kv.get_text_with_slice(&["a", "b"])`, which
returns a `Map<JsString, JsNullable<JsString>>` (a type argument keeps the exact carrier; `get_bulk`
folds it to `HashMap<String, Option<String>>`). Every empty-admitting read is an `Option<T>`
(`kv.get_text(k)` is `Option<String>`, whether the declaration says `null`, `undefined` or
both); only writes and type arguments keep `Nullable` / `JsNullable`. `list()` returns the generated
`KVNamespaceListResult` (`keys()` is a `Vec`, `list_complete()`, `cursor()`).

## Errors

`Error::RustError(s)` is `Error::Rust(s)`; a thrown JS error is `Error::Js(JsException)`,
with `Display` `Name: message` as before (a thrown primitive displays as `String(v)`).
`Result<T, E = Error>`, `ok::Ok`, `From<&str>`/`From<String>` are unchanged.

## Streams

`ByteStream` is unchanged in use: `req.bytes_stream()`, `res.bytes_stream()`,
`Response::builder().from_stream(stream)`. Under the `http` feature it is also the body
type: `HttpRequest = http::Request<ByteStream>`, `HttpResponse = http::Response<ByteStream>`,
with `ByteStream::empty()` and `into_inner()` where `worker::Body` had them.

## Unchanged

`console_log!` and friends, `Delay`, `Cors`, `#[send]` and `send::*`, `Router` (give it
the env type: `Router::<Env>::new()`, `Router::<Env, D>::with_data(d)`), `Method`,
`DateInit` (on `js_sys::Date` through `DateExt`: `Date::current()`, `from_init`,
`as_millis`).
