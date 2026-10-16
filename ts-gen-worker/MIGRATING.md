# Migrating from `worker` to `ts-gen-worker`

The bindings are generated from `@cloudflare/workers-types`, so the JS names are the
Rust names: `webSocket` is `web_socket()`, a `ResponseInit` is built with the generated
`ResponseInit::builder()`, and the overloads of a JS method are `new_with_str`,
`new_with_str_and_init`, and so on. What `worker` added on top of the platform (the
`#[event]` macros, `Result`, serde, `Method`, `Router`, `ByteStream`, Durable Object
classes) is still there as `impl` blocks on the generated types with the same shapes
wherever a name was free. The full row-by-row list is the migration table in
[`parity/crate.md`](parity/crate.md); this guide covers the changes a typical worker meets.

Every sync binding that can throw returns `Result`, so a few calls gain a `?`
(`Headers::new()?`, `ctx.pass_through_on_exception()?`). Awaited values come back as
native types (`text().await?` is a `String`, `status()` is a `u16`).

Arguments spell which empty the JS type admits: `x?: T` takes `Option<&T>`, `T | null`
takes `&js_sys::JsNullable<T>` (`&JsNullable::wrap(v)` or `&JsNullable::new()`). String-enum
options take the generated enum (`EncodeBodyKind::Manual`, `CacheKind::NoStore`,
`BinaryTypeKind::Arraybuffer`), and arrays come back as `Vec<T>`. The JS globals are
lower-case statics: `console`, `caches`, `crypto`, `performance` (`caches.with(CacheStorage::default)`
is `worker`'s `Cache::default()`).

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
* `ctx.wait_until(fut)` is `ctx.spawn(fut)?`; the generated `wait_until(&Promise)` keeps
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
is `.web_socket(&JsNullable::wrap(ws))`, `with_encode_body(EncodeBody::Manual)` is
`.encode_body(EncodeBodyKind::Manual)`, `with_cf(v)?` is
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
    &RequestInit::builder().method("POST").body_with_str(&JsNullable::wrap(b.into())).build(),
)?
```

`with_cache(CacheMode::NoStore)` is `.cache(CacheKind::NoStore)`.

`Fetch::Url(..).send()`, `Fetch::Request(..).send_with_signal(&signal)` and
`AbortController::default()` are unchanged (`Fetch::Url` holds a `String`); the generated
`fetch_with_str(url)` / `fetch(&req)` are the direct forms.

## Headers

`Headers::new()?` or `Headers::default()`; `get`/`has`/`set`/`append`/`delete`/`get_all`
as before; `for (k, v) in &headers` and `[(k, v)].into_iter().collect::<Headers>()` as
before. `keys()`/`values()` are the generated JS iterators. `HeaderMap` converts both
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
is `PartialEq` but not `Eq`, so it cannot key a `HashSet`.

```rust
// worker                                         // ts-gen-worker
env.durable_object("X")?.id_from_name(n)?.get_stub()?   env.x().get_by_name(n)?
stub.fetch_with_request(req).await?               stub.fetch(&req).await?
state.storage().get::<T>(k).await?                storage.get_into::<T>(k).await?
state.storage().put(k, v).await?                  storage.put_serialized(k, &v).await?
```

The generated `storage.get::<JsValue>(k)` / `storage.put(k, &js_value)` take JS handles
directly.

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

`get_bulk`, `put_bytes`, `put_stream` keep their names. `list()` returns the generated
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
