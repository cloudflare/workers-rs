# Generic mono end state: the built prototype (B) vs today (A) vs a non-mono end state (C)

A = non-mono today (`a4cbe97` bindings, `104579e` port). B = the phase-2 prototype: mono +
`js_sys_unstable_apis` + js-sys made fully mono + the ts-gen and wasm-bindgen fixes, built on
`proto/mono` in workers-rs-3 (ts-gen and wasm-bindgen `proto/mono`, all local; every patch is in
`mono-prototype-patches.md`). B1 = phase 1 (`2aa55cc` port on ts-gen `87882e1`, wasm-bindgen
`4fe6881c0`), kept as a column where it shows what phase 2 added. C = non-mono + wasm-bindgen#5167
+ a `JsStr<'a>` either type behind `impl JsStringLike` + tagged enums for primitive unions:
**design, not built**, written by hand from the #5167 compile matrix (`~/Projects/wasm-bindgen-5`
`2497681c1`, stable js-sys).

## Verdict

1. B builds with no stopgaps and passes the suite (194/194, plain and `-http`). Its port has no `JsString`, `String::from(<js>)`, `JsNullable`, `None::<…>` or `Promise::<…>`. Its `unchecked_*` casts fell from 28 (A) to 12, and those sit at `any`-typed or erased boundaries.
2. Everything phase 1 assumed is now built. That covers `T | null` arguments, typed constructors, `Map<_, Option<_>>::get`, `Borrow<Q>` keys, typed `Promise::new`, one import per wire format, and callbacks with Rust leaves. Rust iteration and `Object<T>` as a record came on top.
3. It took 9 wasm-bindgen/js-sys commits and 9 ts-gen commits over phase 1. Under the cfg, js-sys becomes a different API: collections behave like Rust collections. That break, and the `worker`/serde-wasm-bindgen/wasm-streams fallout, is the price of the containers.
4. Costs: COSTS_LINE
5. Mono's own lead over C is unchanged from phase 1: leaf type arguments on generated generics (`get::<f64>`, `first`), primitive unions as one function, and callbacks with Rust leaves. Most other gains are ts-gen lowering or unstable js-sys, which a non-mono build could also adopt.

**Not built** (remaining gaps, §6): KV's typed-options overloads; `Body::body()` typed as a byte
stream; one rule for return-only type parameters; overload names; `{@string}` inside `declare
namespace`; `worker`/`worker-sys` under `js_sys_unstable_apis`, so the 145-test `chomp test` is
still excluded.

## 1. End-state examples

Each A block cites its source at `104579e` (paths under `ts-gen-worker/`; `bindings.rs` lines are
`a4cbe97`). B blocks are the committed prototype port unless marked *signature only*. C blocks are
hand-written.

### Headers (set / get / entries)

```rust
// A: src/ext/cors.rs:62, src/ext/headers.rs:19-26, test/src/ext.rs:195-200
headers.set("Access-Control-Max-Age", &max_age.to_string())?;           // value: &str
pub struct HeadersIter(IntoIter<ArrayTuple<(JsString, JsString)>>);
let (name, value) = self.0.next()?.ok()?.into_tuple();
Some((name.into(), value.into()))
let keys = headers.keys().into_iter().map(|k| String::from(k.unwrap())).collect::<Vec<_>>().join(",");
```
```rust
// B: src/ext/cors.rs:62, src/ext/headers.rs:10-17, test/src/ext.rs:191
headers.set("Access-Control-Max-Age", max_age.to_string())?;            // impl JsStringLike
impl IntoIterator for &Headers {
    type IntoIter = js_sys::Entries<String, String>;                    // yields (String, String)
    fn into_iter(self) -> Self::IntoIter { self.entries().into() }
}
let keys = headers.keys().into_iter().collect::<Vec<_>>().join(",");     // Iterator<String> yields String
```
```rust
// C
headers.set("Access-Control-Max-Age", max_age.to_string())?;            // impl JsStringLike -> JsStr
pub struct HeadersIter(IntoIter<ArrayTuple<(String, String)>>);         // String: JsGeneric (#5167)
Some(self.0.next()?.ok()?.into_tuple())
let keys = headers.keys().into_iter().collect::<Result<Vec<String>, _>>()?.join(",");
```
B gains `String` items with no `Result` per item: a throwing `next()` rethrows, and `try_iter()`
keeps the protocol's `Result`. The hand-written iterator type is gone. `Headers::get` and friends
return `String` only (`{@string copy}`). C keeps the `Result` items and the wrapper.

### Request / Response, init builder, `cf`

```rust
// A: src/ext/interop.rs:84-97, test/src/request.rs:15
let mut init = RequestInit::builder().method(req.method().as_str()).headers(&Headers::try_from(req.headers())?);
init = init.signal(&js_sys::JsNullable::wrap(signal));
init = init.cf(&cf);
init = init.body(body.unchecked_ref());                                  // &JsNullable<ReadableStream<Uint8Array>>
Ok(Request::new_with_str_and_init(&uri, &init.build())?)
let cf: Option<IncomingRequestCfProperties> = req.cf().map(JsCast::unchecked_into);
```
```rust
// B: src/ext/interop.rs:84-97, test/src/request.rs:13
init = init.signal(&signal);                                             // builder takes the bare value
init = init.cf(&cf);
init = init.body(body.unchecked_ref());                                  // &ReadableStream<Uint8Array>
Ok(Request::new_with_str_and_init(&uri, &init.build())?)
let cf = req.cf();                                                       // Option<IncomingRequestCfProperties>
```
```rust
// C: A's form, with `.signal(&signal)` if ts-gen's bare builder values are adopted (not mono-specific).
```
B drops the nullable wrapper on every builder value. The `cf` cast was already unnecessary in A
once `--annotate` set the `Cf` default. One cast remains: `body` is the `http` crate's stream,
which wasm-streams hands over as `web_sys::ReadableStream`.

### Body from string / bytes / stream

```rust
// A: test/src/lib.rs:153-164, src/ext/response.rs:107-118
Response::new_with_array_buffer_and_init(&JsNullable::wrap(out.buffer()), &init)?
Response::new_with_str(&JsNullable::wrap(format!("{a}|{b}").into()))?  // body: &JsNullable<JsString>
Response::new_with_str_and_init(&JsNullable::new(), &init)?             // empty body
Response::new_with_readable_stream_and_init(&JsNullable::wrap(stream.unchecked_into()), &init)?
```
```rust
// B: test/src/lib.rs:148-151, 157, src/ext/response.rs:99-112
Response::new_with_array_buffer_and_init(&out.buffer(), &init)?
Response::new_with_str(format!("{a}|{b}"))?                              // body: impl JsStringLike
Response::new_with_null_and_init(&init)?                                 // the union's null arm
Response::new_with_readable_stream_and_init(stream.unchecked_ref(), &init)?
```
```rust
// C (design): `string | null` as Option<impl JsStringLike>, a nullable handle as Option<&T>
Response::new_with_str(Some(format!("{a}|{b}")))?
Response::new_with_str_and_init(None::<&str>, &init)?
Response::new_with_array_buffer_and_init(Some(&out.buffer()), &init)?
```
An expanded `BodyInit | null` takes each arm bare and emits the `null` arm as its own wrapper, so
neither `Some(…)` nor `None::<&str>` appears. All 44 `JsNullable::wrap/new` are gone. The
remaining cast is `Body::body()`: upstream types it `ReadableStream` (`any` chunks) while every
`BodyInit` slot wants `ReadableStream<Uint8Array>` (§6). C could adopt the same arm lowering:
it is ts-gen, not mono.

### KV (get, metadata, bulk get → `Map`, put options, list)

```rust
// A: test/src/kv.rs:164-176, 181, src/ext/kv.rs:106, test/src/kv.rs:116, 154-158
let keys: Vec<String> = keys.iter().map(|k| (*k).to_owned()).collect();     // key: &[String]
Ok(store.get_with_slice(&keys).await?.unchecked_into())                      // Map<JsString, JsNullable<JsString>>
kv_assert_eq!(values.get(&"bulk_a".into()).as_string(), Some("value_a".to_string()))?;
KVNamespaceGetOptions::builder(&JsString::from(value_type))
KVNamespaceListOptions::builder().prefix(&JsNullable::wrap("put_c".into()))
let result = store.get_with_metadata("put_d").await?;                        // <JsString, JsValue>
```
```rust
// B: test/src/kv.rs:167-174, 180, 112, src/ext/kv.rs:109
let values = store.get_with_slice(&["bulk_a", "bulk_b", "bulk_missing"]).await?; // Map<String, Option<String>>
kv_assert_eq!(values.get("bulk_a"), Some(Some("value_a".to_string())))?;        // Borrow<str> key
kv_assert_eq!(values.get("bulk_missing"), Some(None))?;
let values = store.get_with_slice(&[] as &[&str]).await?;                        // element type needed
KVNamespaceListOptions::builder().prefix("put_c")
KVNamespaceGetOptions::builder(&value_type.to_string())                          // `&str` would infer T = str
```
```rust
// C
let values = store.get_with_slice(&["bulk_a", "bulk_b", "bulk_missing"]).await?;
kv_assert_eq!(values.get(&"bulk_a".to_string()), Some("value_a".to_string()))?;  // stable get -> V; key exactly &String
KVNamespaceListOptions::builder().prefix(Some("put_c"))
```
B reads the bulk result like a `HashMap<String, Option<String>>`: `get` takes a borrowed key and
returns `Option<V>`, so a missing entry and a `null` value stay distinct (`None` vs `Some(None)`),
without a nested-option ABI (`has` + an unchecked read). Phase 1 had to iterate. Still wrong in B:
`&[]` needs its element type, and KV's `get(key, { type })` overloads collapse to one text-typed
form, which forces two casts in `src/ext/kv.rs` (§6).

### R2 (put / get / list, `delimited_prefixes`, custom metadata)

```rust
// A: test/src/r2.rs:75-96, 175
object.custom_metadata().map(|record| {                                      // Option<Object<JsString>>
    Object::entries(record.unchecked_ref()).iter().filter_map(|entry| {
        let entry: js_sys::Array = entry.unchecked_into();
        Some((entry.get(0).as_string()?, entry.get(1).as_string()?))
    }).collect()
}).unwrap_or_default()
fn record(map: &HashMap<String, String>) -> Object<JsString> {
    let obj = Object::new();
    for (k, v) in map { Reflect::set(&obj, &k.into(), &v.into()).unwrap(); }
    obj.unchecked_into()
}
bucket.put_with_str("no-props-no-body", &JsNullable::new()).await?;
R2ListOptions::builder().include(&["customMetadata".to_string()])            // &[String]
```
```rust
// B: test/src/r2.rs:74-79, 113, 154, 360
object.custom_metadata().map(|record| record.iter().collect()).unwrap_or_default() // (String, String)
    .custom_metadata(&custom_metadata.iter().collect())                    // HashMap -> Object<String>
bucket.put_with_null("no-props-no-body").await?;
R2ListOptions::builder().include(&["customMetadata"])
```
```rust
// C
object.custom_metadata()
    .and_then(|record| Object::entries_typed(&record).ok())                  // Array<ArrayTuple<(JsString, String)>>
    .map(|entries| entries.iter().map(|e| { let (k, v) = e.into_tuple(); (String::from(k), v) }).collect())
    .unwrap_or_default()
fn record(map: &HashMap<String, String>) -> Object<String> { /* Reflect::set loop, stable &JsValue keys */ }
```
`Record<string, string>` is now a Rust map in both directions: `iter()` yields `(String, T)` and
`collect()` builds one from any `(key, value)` pairs, so both helpers are gone. C still leaks
`JsString` where js-sys's stable signature fixes the key type.

### D1 (bind mixed values, `first` by column, `all` / `raw`)

```rust
// A: test/src/d1.rs:16-20, 38-40, 57-61, bindings.rs:19788
let bindings = args.iter().map(|arg| arg.serialize(&serializer).map_err(|e| e.to_string()))
    .collect::<Result<Vec<JsValue>, String>>()?;
let value: JsValue = match col_name {
    Some(col) => stmt.first_with_col_name::<JsValue>(col).await?,            // JsNullable<T>
    None => stmt.first::<JsValue>().await?,
}.into();
Ok(serde_wasm_bindgen::from_value(value)?)
// raw_with_d1_prepared_statement_options -> ArrayTuple<(Array<JsString>, Array<Array>)>
```
```rust
// B: test/src/d1.rs:16-19, 57-61; bindings.rs raw signature
let bindings = args.iter().map(|arg| arg.serialize(&serializer)).collect::<Result<Vec<_>, _>>()?;
let value = match col_name {
    Some(col) => stmt.first_with_col_name(col).await?,                      // Option<T>, T = JsValue inferred
    None => stmt.first().await?,
};
Ok(value.map(serde_wasm_bindgen::from_value).transpose()?)
let n: Option<f64> = stmt.first_with_col_name("n").await?;                    // signature only
// raw_with_d1_prepared_statement_options -> ArrayTuple<(Array<String>, Array<Array>)>
```
```rust
// C
Some(col) => stmt.first_with_col_name::<JsValue>(col).await?,                // JsNullable<T>
// first_with_col_name::<f64>("n") does not compile: T inside JsNullable<T> is structural.
```
`first` returns `Option<T>`, so `T` is inferred from its use and the turbofish goes; a leaf `T`
works. C cannot reach a leaf `T` inside the nullable carrier.

### Durable Object storage (get / put / list → `Map`, bulk get, put entries)

```rust
// A: test/src/ext.rs:98-104, test/src/durable.rs:83-99, 168-178
let count = storage.get::<JsValue>("count").await?.as_f64().unwrap_or(0.0) + 1.0; // JsOption<T>
storage.put("count", &JsValue::from(count)).await?;
for key in list.keys() { let key = String::from(key?); /* … */ }             // Map<JsString, JsValue>
let vals: js_sys::Map = storage.get_with_keys(&keys).await?.unchecked_into();
storage.delete_with_keys(&["thing".to_string(), "other".to_string()]).await?;
let obj = js_sys::Object::new();
js_sys::Reflect::set(&obj, &JsValue::from_str("foo"), &value.into())?;
storage.put_with_entries::<JsValue>(&obj.unchecked_into()).await?;
```
```rust
// B: test/src/ext.rs:98-99, test/src/durable.rs:82-96, 161-165
let count = storage.get::<f64>("count").await?.unwrap_or(0.0) + 1.0;       // Option<T>
storage.put("count", &count).await?;
for key in list.keys() { if key != "count" { keys.push(key); } }             // keys() yields String
let vals = storage.get_with_keys(&keys).await?;                              // Map<String, JsValue>
let val = |key| vals.get(key).unwrap_or_default();                           // Borrow<str>
storage.delete_with_keys(&["thing", "other"]).await?;
let entries = js_sys::Object::from_iter([("foo", Uint8Array::from(BAR))]);
storage.put_with_entries(&entries).await?;
```
```rust
// C
let count = storage.get::<JsValue>("count").await?.as_f64().unwrap_or(0.0) + 1.0; // JsOption<T>: no leaf path
storage.put("count", &count).await?;                                        // root &T via ref_to_js
for key in list.keys() { let key = key?; /* … */ }
let val = |key: &str| vals.get(&key.to_string());                            // stable get -> V
js_sys::Reflect::set(&obj, &"foo".into(), &value)?;
```
B's strongest area, as in phase 1: `get::<f64>` and `put(&f64)` need mono. Phase 2 adds the
`Result`-free key iteration and records built by `from_iter`.

### WebSocket hibernation (tags, `get_web_sockets(tag)`, `get_tags`), `WebSocketPair`

```rust
// A: bindings.rs:1207, 1255; test/src/counter.rs:37-43
state.accept_web_socket_with_tags(&server, &["room".to_string()])?;          // tags: &[String]
let tags: Vec<String> = state.get_tags(&ws)?;
let pair = WebSocketPair::new()?;
let server = pair._1();
return Response::builder().from_websocket(pair._0());
```
```rust
// B: test/src/counter.rs:38-43; tag APIs signature only
state.accept_web_socket_with_tags(&server, &["room"])?;                     // &[impl JsStringLike]
let tags = state.get_tags(&ws)?;                                            // Vec<String>, no twin
let (client, server) = WebSocketPair::new()?.into();                        // ext From impl
return Response::builder().from_websocket(client);
```
```rust
// C: B's tag call sites (list either type); the pair is ext code in any design.
```
B and C both accept literal tag lists. `{@string copy}` removed the `get_tags_js_string` twin.
`WebSocketPair`'s `0`/`1` keys lower to `_0()`/`_1()`; a one-line ext `From` gives the tuple.

### WebSocket send / close / message data / event listeners

```rust
// A: test/src/counter.rs:67, test/src/container.rs:119-133, test/src/util.rs:33-42
ws.send_with_str(&format!("{count}"))?;                                      // message: &str
dst.close_with_code_and_reason(reason.code(), &reason.reason())?;
Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| { /* event.unchecked_into() */ })
ws.add_event_listener("message", &Function::from_closure(on_message))?;
event.data().as_string()                                                     // data: any -> JsValue
```
```rust
// B: test/src/counter.rs:63, test/src/container.rs:117-128, test/src/util.rs:31-41
ws.send_with_str(format!("{count}"))?;                                       // impl JsStringLike
dst.close_with_code_and_reason(reason.code(), reason.reason())?;
let on_message = Function::from_fn(move |event: JsValue| { /* event.unchecked_into() */ });
ws.add_event_listener("message", &on_message)?;                              // &Function<fn(JsValue)>
event.data().as_string()
```
```rust
// C: B's send/close call sites; callbacks stay erased (`Function<fn(JsValue) -> Undefined>`).
```
In phase 1, `send_with_str(&s)` and `send_with_str(s)` were two imports. Now `&str`, `String` and
`&String` share one import, borrowed in JS and dropped in Rust after the call, so B's import
count matches A's. Callbacks take Rust leaves with `void` as `()`, so a unit closure fits
`Function::from_fn`. The event itself is still `JsValue`: `addEventListener<K>`'s
`(e: Map[K]) => …` is an indexed access type, so one cast per listener stays.

### Queues (send / sendBatch, message body)

```rust
// A: test/src/queue.rs:45-50, 69-75
my_queue.send(&body).await?;                                                 // message: &Body
let batch = js_sys::Array::new();
for message in &messages { batch.push(&MessageSendRequest::new(&serde_wasm_bindgen::to_value(message)?)); }
my_queue.send_batch(&batch).await?;                                          // messages: &JsValue
```
```rust
// B: test/src/queue.rs:65-69
let batch = messages.iter()
    .map(|message| Ok(MessageSendRequest::new(&serde_wasm_bindgen::to_value(message)?)))
    .collect::<Result<Vec<_>>>()?;
my_queue.send_batch(&batch).await?;                                          // &[MessageSendRequest]
```
```rust
// C: A's form; `Iterable<T>` could lower to `&[T]` there too (ts-gen, not mono).
```
`Iterable<T>` arguments lower like `T[]`, so the batch is a `Vec`.

### Streams (`TransformStream`, writer)

```rust
// A: test/src/cache.rs:125-141, test/src/request.rs:66-92
let transform = TransformStream::new()?;                                     // TransformStream<JsValue, JsValue>
writer.write_with_chunk(&Uint8Array::from(data)).await?;
Response::new_with_readable_stream_and_init(transform.readable().unchecked_ref(), &init)?
Response::new_with_readable_stream(&transform.readable().unchecked_into())?
```
```rust
// B: test/src/cache.rs:120-137, test/src/request.rs:64-89
let transform = TransformStream::new_typed()?;                               // <Uint8Array, Uint8Array> inferred
writer.write_with_chunk(&chunk).await?;                                      // chunk: &Uint8Array fixes I
Response::new_with_readable_stream_and_init(&transform.readable(), &init)?   // fixes O
Response::new_with_readable_stream(&transform.readable())?
```
```rust
// C: A's form; the constructor binds no class parameter.
```
`new()` keeps A's defaults (the `HashMap::new` shape). `new_typed()` binds every class parameter
and infers them from use, so the casts go. Byte-source constructors return
`ReadableStream<Uint8Array>` as the TS declares.

### Cache

```rust
// A: test/src/cache.rs:40-48
let key = req.url();
if let Some(resp) = cache.match_with_str_and_options(&key, &options).await? { /* … */ }
cache.put_with_str(&key, &resp.clone()?).await?;
console.with(|c| c.log(&["Cache HIT!".into()]));
```
```rust
// B: test/src/cache.rs:33-46: identical calls; logs are `console_log!("Cache HIT!")`.
```
```rust
// C: identical.
```
No difference at the bindings. `Request::url` is `String` only (`{@string copy}`).

### Email (attachments, optional strings)

```rust
// A: test/src/send_email.rs:96-103, 133-137; bindings.rs:20152
EmailAttachment::new_attachment_with_typed_array("hello.txt", "text/plain", &bytes);
EmailMessage::new_with_str(from, to, &scenario.raw())?
EmailAttachment::builder(/* … */).content_id(None)                           // val: Option<&str>
```
```rust
// B: test/src/send_email.rs:99, 136; content_id is signature only
EmailMessage::new_with_str(from, to, scenario.raw())?
EmailAttachment::builder(/* … */).content_id("logo")                         // bare; omit the call for none
```
```rust
// C: Option<impl JsStringLike> needs None::<&str>; a concrete Option<JsStr<'_>> needs Some("logo".into()).
```
Phase 1 regressed here (`content_id(None::<&str>)`). Builder methods now take an optional field's
value bare, so there is no `None` to spell. 16 `Option<impl JsStringLike>` arguments remain on raw
setters (`set_content_id(None)` clears a field) and on required nullable dictionary fields.

### Tracing span attributes (primitive union)

Signature only (`Span` is not used by the port).
```rust
// A: bindings.rs:13784-13792
span.set_attribute("cached", true)?;
span.set_attribute_with_f64("retries", 3.0)?;
span.set_attribute_with_str("route", "/api")?;
```
```rust
// B: one function, value: impl js_sys::JsBooleanOrNumberOrStringLike
span.set_attribute("cached", true)?;
span.set_attribute("retries", 3)?;                                           // any int, f64, bool, &str, String
span.set_attribute("route", "/api")?;
```
```rust
// C: value: impl Into<BooleanOrNumberOrString<'_>>, a tagged enum
span.set_attribute("retries", 3.0)?;
```
Unchanged from phase 1. With one import per wire format, each Rust type now costs one shim per
ABI shape rather than one per type.

### Env vars and secrets

```rust
// A: src/ext/env.rs:29-41, test/src/alarm.rs:50-51, test/src/form.rs:139-140
let namespace: DurableObjectNamespace = js_sys::Reflect::get(&env, &"ALARM".into())?.unchecked_into();
let val = js_sys::Reflect::get(&env, &name.into())?;
val.as_string().unwrap_or_default()
```
```rust
// B: test/src/alarm.rs:50, test/src/form.rs:137, test/types/env.d.ts
let namespace: DurableObjectNamespace = env.get("ALARM")?;                  // EnvExt
return Response::builder().ok(env.secret(&name).unwrap_or_default());
env.some_secret()                                                           // String; Env is readonly + {@string copy}
```
```rust
// C: identical to B (EnvExt is ext code).
```
The `Reflect` detours were port debt, not binding gaps: `EnvExt` already covered them. The
generated `Env` has no setters and no string twins.

### FormData / URLSearchParams

```rust
// A: test/src/form.rs:17-22, 35, 39-47
fn from_js(value: JsValue) -> Option<FormEntry> {                            // get: File | string | null -> JsValue
    if let Some(s) = value.as_string() { Some(FormEntry::Field(s)) }
    else { value.dyn_into::<File>().ok().map(FormEntry::File) }
}
```
```rust
// B: identical (test/src/form.rs:16-22). `forEach` callbacks are now
// Function<fn(JsValue, String, FormData)> (signature only).
```
```rust
// C: identical; `File | string` stays erased.
```
A handle-or-string union needs a ts-gen enum in any mode.

### js-sys built-ins (`Error`, `Date`, `Promise`)

```rust
// A: src/error.rs:89-94, test/src/user.rs:36-43, test/src/util.rs:11-16, test/src/socket.rs:97
write!(f, "{}: {}", String::from(e.name()), String::from(e.message()))
js_sys::Date::new(&1_234_567_890.0.into()).to_string().into()
let promise = Promise::new(&mut |resolve: Function, _| { let _ = set_timeout_with_function_and_ms_delay(&resolve, ms); });
let _ = JsFuture::from(promise).await;
JsFuture::from(socket.opened()).await?;
```
```rust
// B: src/error.rs:89, test/src/user.rs:36, test/src/util.rs:10-12, test/src/socket.rs:96
Some(e) => write!(f, "{}: {}", e.name(), e.message()),                      // String
js_sys::Date::new_with_value(1_234_567_890.0).to_string()
Delay::from(Duration::from_millis(ms as u64)).await;                        // the crate's timer
socket.opened().await?;                                                     // Promise<T>: IntoFuture
```
```rust
// C: A's form (stable js-sys).
```
Small-string returns of js-sys built-ins are `String`. `Promise::new` is typed
(`Function<fn(T)>` resolver); the port no longer needs it.

## 2. Port tally, A vs B

Regex counts over `git show <rev>:` of `ts-gen-worker/{src,test/src,benchmark/src}`, excluding the
generated `bindings.rs` and `env.rs` and skipping comment lines.

TALLY_TABLE

## 3. API diff

Type-level counts in the generated `bindings.rs` (doc and comment lines skipped):

BINDINGS_TABLE

Every item that is **still worse in B than in A**:
- **`&[String]` → `&[impl JsStringLike]` (117):** `&[]` needs `&[] as &[&str]`. Literal lists and `&Vec<String>` both work.
- **`Option<&str>` → `Option<impl JsStringLike>` (16):** only on raw setters and required nullable fields. `None` there needs `None::<&str>`; builders and expanded unions no longer have it.
- **Generic user code over a helper** must restate the helper's `for<'a> &'a T: IntoWasmAbi` bounds (41 where-clauses, down from 66), where A needed `T: JsGeneric`.
- **`_js_string` twins (+70):** 64 sit in `declare namespace TailStream`, which `--annotate` cannot reach. The other 6 are deliberate (`text()`, KV values).
- **Surface outside `bindings.rs` (unstable js-sys):** `Reflect::get` → `Result<Option<T>>`, `Array::get` → `Option<T>`, and iterators that rethrow instead of yielding `Result` (`try_iter()` keeps the protocol). Code written against stable js-sys does not compile under the cfg (`worker`, `worker-sys`, serde-wasm-bindgen, wasm-streams).

## 4. Costs

COSTS_SECTION

## 5. "Fleshed out" checklist

Every change B needed, all built (see `mono-prototype-patches.md` for commits and files):

| change | where | phase |
|---|---|---|
| `js_sys_unstable_apis` for wasm32; serde-wasm-bindgen and wasm-streams patched copies | workers-rs | 1 |
| native leaves inside js-sys containers; `&T` generic arguments; constructor defaults; awaited `Option<T>` | ts-gen | 1 |
| feedback 8: argument-position `impl Trait` keeps declared generic arity | wasm-bindgen macro | 1 |
| `ArrayTuple`/`Object::entries` Rust-typed | js-sys | 1 |
| one import per wire format (`&str`/`String`/`&String`, handles by value or reference); no `// generic import` comments; `?Sized` mono params; `&Option<T>`, `Option<&T>` arguments | wasm-bindgen macro + CLI | 2 |
| `Map` by `Borrow<Q>`, `Map::get` for every `V` (`Option<Option<V>>` without a nested ABI) | js-sys | 2 |
| `Function<fn(A..) -> R>` with any ABI `A`/`R`, `()` for void; `Function::from_fn`; typed `Promise::new`; `From<Closure>` | js-sys | 2 |
| `String` returns from built-ins; `Date::new_with_value` | js-sys | 2 |
| iterators yield `T`; `Entries<K, V>`; Map/Set/Object as Rust collections (`iter`, `FromIterator`, `Extend`) | js-sys | 2 |
| stopgaps #1–#5 replaced by ts-gen fixes (`mono-patch` deleted) | ts-gen | 2 |
| `{@string copy\|ref\|both}`; twin naming | ts-gen + `annotations.d.ts` | 2 |
| `_typed` constructor twins; concrete construct-signature returns | ts-gen | 2 |
| `T \| null` arguments: bare union arms + `_with_null` arm; `Option<…>` elsewhere; bare builder values | ts-gen | 2 |
| callbacks with Rust leaves, `void` → `()`, `Promise<()>` | ts-gen | 2 |
| `Iterable<T>` → slices; native alias leaves; optional `any` → `Option<JsValue>` | ts-gen | 2 |

C would need instead (none built): #5167 with the root `&T` slot leak fixed; `JsStr<'a>` and a
sealed `JsStringLike::as_js_str` plus a list either type; tagged enums with `From` impls for the
13 primitive unions; hand-written `IntoJsGeneric`/`FromJsGeneric` for every user type used as a
type argument. Most of B's ts-gen lowering (nullable arms, bare builder values, `Iterable` slices,
constructor twins, `{@string}`) is not mono-specific and would carry over. The js-sys collection
behaviour would too, with the same ecosystem break.

## 6. Remaining gaps

GAPS_SECTION
