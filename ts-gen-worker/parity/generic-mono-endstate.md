# Generic mono end state: the built prototype (B) vs today (A) vs a non-mono end state (C)

A = non-mono today (`a4cbe97` bindings, `104579e` port). B = the phase-2 prototype: mono +
`js_sys_unstable_apis` + js-sys made fully mono + the ts-gen and wasm-bindgen fixes, built on
`proto/mono` in workers-rs-3 (ts-gen and wasm-bindgen `proto/mono`, all local; every patch is in
`mono-prototype-patches.md`). B1 = phase 1 (`2aa55cc` port), kept as a column where it shows what
phase 2 added. C = non-mono + wasm-bindgen#5167 + a `JsStr<'a>` either type behind
`impl JsStringLike` + tagged enums for primitive unions: **design, not built**, written by hand
from the #5167 compile matrix (`~/Projects/wasm-bindgen-5` `2497681c1`, stable js-sys).

## Verdict

1. B builds as generated, with no stopgaps, and passes the suite (194/194, plain and `-http`). The port has none of: `JsString`, `String::from(<js>)`, `JsNullable`, `None::<…>`, `map_err(|e| e.to_string())`, `JsFuture::from` at call sites. `unchecked_*` casts went from 29 (A) to 4. Each of the 4 sits at an `any`-typed or erased boundary (§2).
2. Everything phase 1 assumed is built, and the R audit's items are done except the ones in §6. Phase 2 also built Rust iteration, `Object<T>` as a map, typed KV reads, byte-stream bodies, one argument for buffer sources, one `Promise<T>` per async member, and shorter overload names.
3. The cost in commits: 14 wasm-bindgen/js-sys commits and 27 ts-gen commits on top of phase 1. Under the cfg, js-sys is a different API in which collections behave like Rust collections. That API break, and the fallout in `worker`, serde-wasm-bindgen and wasm-streams, is the price of the containers. `worker` is not ported (§6).
4. The cost in size and time: benchmark wasm +586 (+0.5%), test glue +9,149 (+5.2%) (phase 1: +12.7%), test imports +18 (phase 1: +16), 0 generic shims (phase 1: 177). `ts-gen-worker` compiles +5% debug and +7% release (§4).
5. Mono's own lead over C is unchanged from phase 1:
   - leaf type arguments on generated generics (`get::<f64>`, `get_with_metadata_typed::<f64>`, `first`);
   - primitive unions as one function;
   - callbacks with Rust leaves.

   Buffer-source unions are only partly a mono lead: the handle forms share one wire format, so C could take them
   through an inline wrapper over `&JsValue`. Only the `&[u8]` view and the `Vec<u8>` copy need per-instantiation
   lowering.

   Most other gains are ts-gen lowering or unstable js-sys, which a non-mono build could also adopt (§5).
6. Lowering policies chosen while building B, each reversible in ts-gen alone, are listed in §7 for review.

## 1. End-state examples

A blocks cite their source at `104579e`, with paths under `ts-gen-worker/` and `bindings.rs` lines
at `a4cbe97`. B blocks are the committed prototype port unless marked *signature only*. C blocks
are hand-written.

### Headers (set / get / iterate)

```rust
// A: src/ext/cors.rs:62, src/ext/headers.rs:19-26, test/src/ext.rs:195-200
headers.set("Access-Control-Max-Age", &max_age.to_string())?;           // value: &str
pub struct HeadersIter(IntoIter<ArrayTuple<(JsString, JsString)>>);
let (name, value) = self.0.next()?.ok()?.into_tuple();
Some((name.into(), value.into()))
let keys = headers.keys().into_iter().map(|k| String::from(k.unwrap())).collect::<Vec<_>>().join(",");
```
```rust
// B: src/ext/cors.rs:62, test/src/ext.rs:185-190; IntoIterator generated from [Symbol.iterator]
headers.set("Access-Control-Max-Age", max_age.to_string())?;            // impl JsStringLike
for (name, value) in &headers { /* … */ }                                // (String, String)
let keys = headers.keys().into_iter().collect::<Vec<_>>().join(",");     // Iterator<String> yields String
```
```rust
// C
headers.set("Access-Control-Max-Age", max_age.to_string())?;            // impl JsStringLike -> JsStr
pub struct HeadersIter(IntoIter<ArrayTuple<(String, String)>>);         // String: JsGeneric (#5167)
Some(self.0.next()?.ok()?.into_tuple())
let keys = headers.keys().into_iter().collect::<Result<Vec<String>, _>>()?.join(",");
```
- B has no hand-written iterator. ts-gen emits `IntoIterator for &Headers` (also for `FormData`, `URLSearchParams` and `SqlStorageCursor<T>`) from the declaration's `[Symbol.iterator]()`.
- Items are `String`, with no per-item `Result`: a throwing `next()` rethrows, and `try_iter()` keeps the protocol's `Result` where it matters (`test/src/sql.rs:111`).
- `Headers::get` and similar return `String` only (`{@string copy}`).

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
// B: src/ext/interop.rs:84-97, test/src/request.rs:11
init = init.signal(&signal);                                             // builder takes the bare value
init = init.cf(&cf);
init = init.body(&body);                                                 // body: ReadableStream<Uint8Array>
Ok(Request::new_with_str_and_init(&uri, &init.build())?)
let cf = req.cf();                                                       // Option<IncomingRequestCfProperties>
```
```rust
// C: A's form, plus `.signal(&signal)` if ts-gen's bare builder values are adopted (not mono-specific).
```
- B drops the nullable wrapper on every builder value.
- The body cast is gone because `readable_stream_from` and `ByteStream` both speak `ReadableStream<Uint8Array>`.
- `Response::cf()` is `Option<JsValue>`, the optional-`any` rule, so the `is_undefined()` checks went too.

### Body from string / bytes / stream

```rust
// A: test/src/lib.rs:153-164, src/ext/response.rs:107-118
Response::new_with_array_buffer_and_init(&JsNullable::wrap(out.buffer()), &init)?
Response::new_with_str(&JsNullable::wrap(format!("{a}|{b}").into()))?  // body: &JsNullable<JsString>
Response::new_with_str_and_init(&JsNullable::new(), &init)?             // empty body
Response::new_with_readable_stream_and_init(&JsNullable::wrap(stream.unchecked_into()), &init)?
```
```rust
// B: test/src/lib.rs:142-158, src/ext/response.rs:97-110
Response::new_with_buffer_source_and_init(&bytes[..], &init)?            // bytes: impl BufferSourceLike
Response::new_with_str(format!("{a}|{b}"))?                              // body: impl JsStringLike
Response::new_with_null_and_init(&init)?                                 // the union's null arm
Response::new_with_readable_stream_and_init(&stream, &init)?             // stream: ReadableStream<Uint8Array>
```
```rust
// C (design): `string | null` as Option<impl JsStringLike>, a nullable handle as Option<&T>
Response::new_with_str(Some(format!("{a}|{b}")))?
Response::new_with_str_and_init(None::<&str>, &init)?
Response::new_with_array_buffer_and_init(Some(&out.buffer()), &init)?
```
- An expanded `BodyInit | null` takes each arm bare and emits the `null` arm as its own wrapper, so neither `Some(…)` nor `None::<&str>` appears. All 44 `JsNullable::wrap/new` are gone.
- `ArrayBuffer | ArrayBufferView` is one argument, `impl js_sys::BufferSourceLike`, a sealed trait like the primitive-union ones. The `_array_buffer` / `_typed_array` arm pairs are gone (crypto `verify` had 8 arms). It takes three forms:
  - buffer handles (`&ArrayBuffer`, typed arrays, `&DataView`) pass as themselves;
  - `Vec<u8>` passes as a copy;
  - `&[u8]`, the form the port uses (`&bytes[..]`), is a view into wasm memory that is valid only for the call. That is right where the callee copies before returning: workerd's KV and R2 `put`, and `Response` by the Fetch spec.
- A dictionary setter stores its value, so it takes `slice = "copy"` and a `&[u8]` arrives as a fresh `Uint8Array` (15 setters: `R2PutOptions::md5`, crypto `iv`/`salt`, `RequestInit` body, attachment content). `test/src/r2.rs:349-356` zeroes the digest after `.md5(&hash[..])` and R2 still verifies it.
- A callee that transfers the buffer (`ReadableByteStreamController::enqueue`) throws `TypeError` on a `&[u8]` view, because wasm memory cannot be detached (§6).
- `Body::body()` is `Option<ReadableStream<Uint8Array>>` through an `@retype` in `annotations.d.ts`. Upstream types it as `ReadableStream`, with `any` chunks.
- C could adopt the same arm lowering, since it is ts-gen, not mono.

### KV (get, typed reads, metadata, bulk get → `Map`, put options, list)

```rust
// A: src/ext/kv.rs:166-184, test/src/kv.rs:90-92, 115, 164-177
let promise: Promise<JsValue> = self.kv
    .get_with_key_str_and_options_kv_namespace_get_options_promise(&self.key, &options)?
    .unchecked_into();                                                       // get(k, {type}) collapses to text
let entry: Entry = entry.unchecked_into();                                   // getWithMetadata the same
kv_assert_eq!(result.metadata().unwrap().as_f64().unwrap() as usize, 100)?;  // Metadata erased to JsValue
let keys: Vec<String> = keys.iter().map(|k| (*k).to_owned()).collect();      // key: &[String]
Ok(store.get_with_slice(&keys).await?.unchecked_into())                      // Map<JsString, JsNullable<JsString>>
kv_assert_eq!(values.get(&"bulk_a".into()).as_string(), Some("value_a".to_string()))?;
KVNamespaceListOptions::builder().prefix(&JsNullable::wrap("put_c".into()))
```
```rust
// B: src/ext/kv.rs:114-121, 158-175, test/src/kv.rs:88-90, 163-174, 112
let options = KVNamespaceGetOptionsPartial::builder().cache_ttl(ttl).build();  // Partial<X>: all fields optional
Ok(self.kv.get_text_with_options(&self.key, &options).await?)                  // sets type: "text"; Option<String>
self.kv.get_array_buffer_with_options(&self.key, &options).await?.map(Vec::from)
let result = store.get_with_metadata_typed::<f64>("put_b").await?;           // nested return-only param
kv_assert_eq!(result.metadata(), Some(100.0))?;
let values = store.get_with_slice(&["bulk_a", "bulk_b", "bulk_missing"]).await?; // Map<String, Option<String>>
kv_assert_eq!(values.get("bulk_a"), Some(Some("value_a".to_string())))?;        // Borrow<str> key
kv_assert_eq!(values.get("bulk_missing"), Some(None))?;
let values = store.get_with_slice(&[] as &[&str]).await?;                        // element type needed
KVNamespaceListOptions::builder().prefix("put_c")
```
```rust
// C
let values = store.get_with_slice(&["bulk_a", "bulk_b", "bulk_missing"]).await?;
kv_assert_eq!(values.get(&"bulk_a".to_string()), Some("value_a".to_string()))?;  // stable get -> V; key exactly &String
KVNamespaceListOptions::builder().prefix(Some("put_c"))
// typed reads: the same per-literal bindings could be emitted (ts-gen); `get_with_metadata::<f64>` cannot.
```
- KV's `get(key, "text" | { type: "text" })` overloads are bindings of their own (`get_text_with_options`, `get_json_with_options::<T>`, `get_array_buffer_with_options`, `get_stream_with_options`, and the same for `getWithMetadata`).
  - Each one is a wrapper that sets `type` to its literal, so the runtime result always matches the return type.
  - The options are `Partial<KVNamespaceGetOptions>`, which lowers to a synthesized all-optional `KVNamespaceGetOptionsPartial`.
  - `src/ext/kv.rs` has no casts and no type literals.
- `getWithMetadata<Metadata>` has a `_typed::<M>` twin, because `Metadata` is a return-only parameter nested in the result. The plain form keeps `JsValue`.
- The bulk result reads like a `HashMap<String, Option<String>>`. `get` takes a borrowed key and returns `Option<V>`, so a missing entry (`None`) and a `null` value (`Some(None)`) stay distinct.
- Still wrong in B: `&[]` needs its element type.

### R2 (put / get / list, custom metadata)

```rust
// A: test/src/r2.rs:75-96, 125, 175
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
// B: test/src/r2.rs:74-79, 105, 154, 352
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
- `Record<string, string>` is a Rust map in both directions: `iter()` yields `(String, T)`, and `collect()` builds one from any `(key, value)` pairs. Both helpers are gone.
- `FixedLengthStream` derefs to `TransformStream<JsValue, Uint8Array>`, so its `readable()` passes to `put` with no cast.
- C still leaks `JsString` where js-sys's stable signature fixes the key type.

### D1 (bind mixed values, `first` by column, `all` / `raw`, `dump`)

```rust
// A: test/src/d1.rs:16-20, 57-62, 262
let bindings = args.iter().map(|arg| arg.serialize(&serializer).map_err(|e| e.to_string()))
    .collect::<Result<Vec<JsValue>, String>>()?;
let value: JsValue = match col_name {
    Some(col) => stmt.first_with_col_name::<JsValue>(col).await?,            // JsNullable<T>
    None => stmt.first::<JsValue>().await?,
}.into();
Ok(serde_wasm_bindgen::from_value(value)?)
let bytes = Uint8Array::new(&db.dump().await?.into()).to_vec();
```
```rust
// B: test/src/d1.rs:16-19, 57-61, 257
let bindings = args.iter().map(|arg| arg.serialize(&serializer)).collect::<Result<Vec<_>, _>>()?;
let value = match col_name {
    Some(col) => stmt.first_with_col_name(col).await?,                      // Option<T>, T = JsValue inferred
    None => stmt.first().await?,
};
Ok(value.map(serde_wasm_bindgen::from_value).transpose()?)
let bytes = db.dump().await?.to_vec();                                      // ArrayBuffer -> Vec<u8>
let n: Option<f64> = stmt.first_with_col_name("n").await?;                    // signature only
```
```rust
// C
Some(col) => stmt.first_with_col_name::<JsValue>(col).await?,                // JsNullable<T>
// first_with_col_name::<f64>("n") does not compile: T inside JsNullable<T> is structural.
```
- `first` returns `Option<T>`, so `T` is inferred from use and the turbofish goes. A leaf `T` works. `all_typed::<T>`, `raw_typed::<T>` and `run_typed::<T>` cover D1's nested return-only parameters.
- C cannot reach a leaf `T` inside the nullable carrier.

### Durable Object storage (get / put / list → `Map`, bulk get, put entries)

```rust
// A: test/src/ext.rs:98-104, test/src/durable.rs:83-99, 144-182
let count = storage.get::<JsValue>("count").await?.as_f64().unwrap_or(0.0) + 1.0; // JsOption<T>
storage.put("count", &JsValue::from(count)).await?;
for key in list.keys() { let key = String::from(key?); /* … */ }             // Map<JsString, JsValue>
let vals: js_sys::Map = storage.get_with_keys(&keys).await?.unchecked_into();
storage.delete_with_keys(&["thing".to_string(), "other".to_string()]).await?;
storage.put_with_entries::<JsValue>(&serde_wasm_bindgen::to_value(&stuff)?.unchecked_into()).await?;
let obj = js_sys::Object::new();
js_sys::Reflect::set(&obj, &JsValue::from_str("foo"), &value.into())?;
```
```rust
// B: test/src/ext.rs:98-99, test/src/durable.rs:82-97, 134-163
let count = storage.get::<f64>("count").await?.unwrap_or(0.0) + 1.0;       // Option<T>
storage.put("count", &count).await?;
for key in list.keys() { if key != "count" { keys.push(key); } }             // keys() yields String
let vals = storage.get_with_keys(&keys).await?;                              // Map<String, JsValue>
let val = |key| vals.get(key).unwrap_or_default();                           // Borrow<str>
storage.delete_with_keys(&["thing", "other"]).await?;
storage.put_entries_serialized(&stuff).await?;                               // ext, beside put_serialized
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
- This is B's strongest area, as in phase 1: `get::<f64>` and `put(&f64)` need mono.
- Phase 2 adds `Result`-free key iteration, records built by `from_iter`, and `get_with_keys_typed::<T>` for a uniform value type.

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
// B: test/src/counter.rs:38-42; tag APIs signature only
state.accept_web_socket_with_tags(&server, &["room"])?;                     // &[impl JsStringLike]
let tags = state.get_tags(&ws)?;                                            // Vec<String>, no twin
let (client, server) = WebSocketPair::new()?.into();                        // src/ext/websocket.rs
return Response::builder().from_websocket(client);
```
```rust
// C: B's tag call sites (list either type); the pair is ext code in any design.
```
- B and C both accept literal tag lists.
- `WebSocketPair` lowers its `0`/`1` keys to `_0()`/`_1()`. A one-line ext `From` impl gives the tuple.

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
// B: test/src/counter.rs:66, test/src/container.rs:128, test/src/util.rs:31-41
ws.send_with_str(format!("{count}"))?;                                       // impl JsStringLike
dst.close_with_code_and_reason(reason.code(), reason.reason())?;
let on_message = Function::from_fn(move |event: MessageEvent| { .. });
ws.add_event_listener_message(&on_message)?;                                 // &Function<fn(MessageEvent)>
event.data().as_string()
```
```rust
// C: B's send/close call sites; callbacks stay erased (`Function<fn(JsValue) -> Undefined>`).
```
- `&str`, `String` and `&String` share one import, borrowed in JS and dropped in Rust after the call. Phase 1 had one import per Rust type.
- Callbacks take Rust leaves, with `void` as `()`, so a unit closure fits `Function::from_fn`.
- `addEventListener<K extends keyof M>(type: K, handler: (e: M[K]) => …)` on `EventTarget<WebSocketEventMap>` binds once per key on the concrete subclass: `add_event_listener_message(&Function<fn(MessageEvent)>)`, `add_event_listener_close(..)`. The generic string form stays beside them, so the casts are gone (D2, §7).

### Queues (send / sendBatch)

```rust
// A: test/src/queue.rs:69-75
let batch = js_sys::Array::new();
for message in &messages { batch.push(&MessageSendRequest::new(&serde_wasm_bindgen::to_value(message)?)); }
my_queue.send_batch(&batch).await?;                                          // messages: &JsValue
```
```rust
// B: test/src/queue.rs:65-74
let batch = messages.iter()
    .map(|message| Ok(MessageSendRequest::new(&serde_wasm_bindgen::to_value(message)?)))
    .collect::<Result<Vec<_>>>()?;
my_queue.send_batch(&batch).await?;                                          // &[MessageSendRequest]
```
```rust
// C: A's form; `Iterable<T>` could lower to `&[T]` there too (ts-gen, not mono).
```

### Streams (`TransformStream`, writer, reader)

```rust
// A: test/src/cache.rs:125-141, test/src/request.rs:66-92, test/src/socket.rs:20
let transform = TransformStream::new()?;                                     // TransformStream<JsValue, JsValue>
writer.write_with_chunk(&Uint8Array::from(data)).await?;
Response::new_with_readable_stream_and_init(transform.readable().unchecked_ref(), &init)?
let bytes = Uint8Array::new(&chunk.value().unwrap_or_default()).to_vec();   // chunk: JsValue
```
```rust
// B: test/src/cache.rs:120-137, test/src/request.rs:64-88, test/src/socket.rs:22
let transform = TransformStream::new_typed()?;                               // <Uint8Array, Uint8Array> inferred
writer.write_with_chunk(&chunk).await?;                                      // chunk: &Uint8Array fixes I
Response::new_with_readable_stream_and_init(&transform.readable(), &init)?   // fixes O
let bytes = chunk.value().unwrap_or_default().to_vec();                     // Socket streams: @retype Uint8Array
```
```rust
// C: A's form; the constructor binds no class parameter.
```
- `new()` keeps A's defaults, the `HashMap::new` shape. `new_typed()` binds every class parameter and infers them from use.
- Bare `ReadableStream` arguments take any instantiation (`&ReadableStream<impl Sized>`), so a typed stream passes to KV/R2 `put` without `.upcast()`.

### Cache

```rust
// A: test/src/cache.rs:40-48
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

### Email (attachments, optional strings)

```rust
// A: test/src/send_email.rs:133-137; bindings.rs:20152
EmailMessage::new_with_str(from, to, &scenario.raw())?
EmailAttachment::builder(/* … */).content_id(None)                           // val: Option<&str>
```
```rust
// B: test/src/send_email.rs:138; content_id is signature only
EmailMessage::new_with_str(from, to, scenario.raw())?
EmailAttachment::builder(/* … */).content_id("logo")                         // bare; omit the call for none
```
```rust
// C: Option<impl JsStringLike> needs None::<&str>; a concrete Option<JsStr<'_>> needs Some("logo".into()).
```
Phase 1 regressed here with `content_id(None::<&str>)`. Builder methods now take an optional
field's value bare, so there is no `None` to spell. The raw setters keep `Option`, so `None` there
still needs `None::<&str>` (§3).

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

### Overload and synthesized names

```rust
// A: bindings.rs (a4cbe97); src/ext/fetch.rs, src/ext/delay.rs, src/ext/kv.rs, test/src/r2.rs:209, 392
fetch_with_request_and_init(&req, &init)
set_timeout_with_function_and_ms_delay(&callback, ms)
kv.get_with_key_str_and_options_kv_namespace_get_options_promise(&key, &options)
kv.put_with_str_and_options(key, value, &options)
bucket.get_with_r2_get_options(key, &options)
bucket.put_with_str_and_r2_put_options(key, value, &options)              // beside put_with_str_and_r2_bucket_options2
```
```rust
// B: src/ext/fetch.rs:26, src/ext/delay.rs:54, src/ext/kv.rs:160, test/src/kv.rs:79, test/src/r2.rs:184, 358
fetch_with_init(&req, &init)                                               // the primary arm (Request) is implied
set_timeout_with_ms_delay(&callback, ms)
kv.get_text_with_options(&key, &options)                                   // one binding per `type` literal
kv.put_with_options(key, value, &options)                                  // a dictionary arm is named by its parameter
bucket.get_with_options(key, &options)
bucket.put_with_str_and_options(key, value, &options)                      // inline `onlyIf` form: put_with_str_and_r2_bucket_put_options
```
Public names over 50 characters fell from 109 to 34. Most of the rest are
`IncomingRequestCfProperties`' numbered builders and spelled-out multi-parameter arms
(`add_event_listener_with_event_listener_object_and_options`). Every rule here is ts-gen naming,
so C could adopt it too.

### Env vars and secrets

```rust
// A: test/src/alarm.rs:50-51, test/src/form.rs:139-140
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
The `Reflect` detours were port debt rather than binding gaps. The generated `Env` has no
setters and no string twins.

### FormData

```rust
// A, B: test/src/form.rs:17-22 (identical)
fn from_js(value: JsValue) -> Option<FormEntry> {                            // get: File | string | null -> JsValue
    if let Some(s) = value.as_string() { Some(FormEntry::Field(s)) }
    else { value.dyn_into::<File>().ok().map(FormEntry::File) }
}
```
A handle-or-string union needs a ts-gen enum in any mode. B's `for (name, value) in &form`
yields `(String, JsValue)`.

### js-sys built-ins (`Error`, `Date`, `JSON`, promises, bytes)

```rust
// A: src/error.rs:89-94, test/src/user.rs:36-43, test/src/util.rs:11-16, test/src/ext.rs:431
write!(f, "{}: {}", String::from(e.name()), String::from(e.message()))
js_sys::Date::new(&1_234_567_890.0.into()).to_string().into()
let promise = Promise::new(&mut |resolve: Function, _| { let _ = set_timeout_with_function_and_ms_delay(&resolve, ms); });
let _ = JsFuture::from(promise).await;
let cf: String = js_sys::JSON::stringify(&res.cf())?.into();
```
```rust
// B: src/error.rs:89, test/src/user.rs:36, test/src/util.rs:12, test/src/ext.rs:416
Some(e) => write!(f, "{}: {}", e.name(), e.message()),                      // String
js_sys::Date::new_with_value(1_234_567_890.0).to_string()
Delay::from(Duration::from_millis(ms as u64)).await;                        // the crate's timer
let cf = js_sys::JSON::stringify(&res.cf().unwrap_or_default())?.unwrap_or_default(); // Option<String>
```
```rust
// C: A's form (stable js-sys).
```
`Promise<T>` awaits directly, `Promise::new` has a typed resolver, and `ArrayBuffer` converts to
and from bytes (`buf.to_vec()`, `ArrayBuffer::from(&bytes[..])`).

## 2. Port tally, A vs B

Regex counts over `git show <rev>:` of `ts-gen-worker/{src,test/src,benchmark/src}`, excluding the
generated `bindings.rs` and the test and benchmark `env.rs`, and skipping comment lines.

| pattern | A `104579e` | B1 `2aa55cc` | B |
|---|---:|---:|---:|
| `JsString` (any mention) | 12 | 0 | 0 |
| `JsString::from` | 2 | 0 | 0 |
| `String::from(<js>)` | 12 | 6 | 0 |
| `.as_string()` | 14 | 9 | 7 |
| `unchecked_into` / `unchecked_ref` | 29 | 19 | 4 |
| `dyn_into` / `dyn_ref` | 7 | 7 | 6 |
| turbofish `::<JsValue` | 9 | 7 | 1 |
| all turbofish `::<` | 92 | 91 | 84 |
| `let x: T =` annotations | 85 | 80 | 75 |
| `None::<…>` | 0 | 0 | 0 |
| `&[] as &[…]` | 0 | 1 | 1 |
| `JsValue::from` / `from_str` / `from_f64` | 29 | 22 | 21 |
| `JsNullable::wrap` / `new` | 44 | 44 | 0 |
| `Promise::<…>` | 0 | 1 | 0 |
| `Reflect::` | 13 | 13 | 2 |
| `map_err(\|e\| e.to_string())` | 21 | 21 | 0 |
| `console.with(…)` | 19 | 19 | 7 |
| `JsFuture::from` | 6 | 6 | 1 |
| `Closure::` | 4 | 4 | 1 |
| `._0()` / `._1()` | 10 | 10 | 2 |
| `Uint8Array::new(&…)` re-wraps | 5 | 5 | 0 |
| lines | 7,444 | 7,386 | 7,217 |

The 4 remaining casts:
- the wasm-streams boundary (`web_sys::ReadableStream` → `ReadableStream<Uint8Array>`, `src/ext/streams.rs:161`);
- `EnvExt::get`, which is unchecked on purpose (`src/ext/env.rs:42`);
- `HeadersInit` read back from a `ResponseInit` (`src/ext/response.rs:37`);
- a serialized struct passed as an entries object (`src/ext/serde.rs:61`).

The `.as_string()` / `dyn_*` calls are on `any` values: `MessageEvent::data`, attachments,
FormData `File | string`, SQL values and `Error::cause`. No `::<JsValue>` is left on a binding:
`Function::once` gives `block_concurrency_while` its `T` from the closure's return.

## 3. API diff

`tools/api-diff` over the generated `bindings.rs` (item-keyed signatures):

- **B vs A**: 1403 items changed, 585 removed, 378 added, 3067 unchanged. Most removed/added pairs are renames (overload suffixes, `_with_null` arms, KV typed reads).
- **B vs B1**: 498 changed, 946 removed (376 of them `_js_string` twins), 368 added, 3982 unchanged.

Type-level counts (doc and comment lines skipped):

| pattern | A `a4cbe97` | B1 `2aa55cc` | B |
|---|---:|---:|---:|
| `JsString` (word) | 175 | 422 | 21 |
| `_js_string` twins | 0 | 383 | 15 |
| `Map<JsString, …>` | 44 | 0 | 0 |
| `Array<JsString>` | 3 | 18 | 0 |
| `Promise<JsString>` | 8 | 0 | 3 |
| `Object<JsString>` | 28 | 0 | 0 |
| `Iterator<JsString>` | 6 | 0 | 0 |
| `JsNullable<JsString>` | 39 | 24 | 0 |
| `JsNullable<…>` | 136 | 91 | 0 |
| `JsOption<…>` | 110 | 95 | 0 |
| `Function<fn(…JsString…)>` | 8 | 8 | 0 |
| `Function<fn(…)>` | 101 | 101 | 127 |
| `-> Undefined` | 17 | 17 | 0 |
| `Promise<Undefined>` | 163 | 162 | 0 |
| `Promise<()>` | 0 | 0 | 160 |
| `Boolean` / `Number` | 32 | 8 | 0 |
| `impl JsStringLike` | 0 | 1,268 | 1,151 |
| `Option<impl JsStringLike>` | 0 | 7 | 15 |
| `&[impl JsStringLike]` | 0 | 117 | 90 |
| `impl js_sys::Js…Like` (primitive unions) | 0 | 13 | 12 |
| `Option<&str>` / `&[String]` | 124 | 0 | 0 |
| `: JsGeneric` bounds | 666 | 75 | 0 |
| `for<'a>` where-clauses | 0 | 66 | 12 |
| `_typed` fns | 0 | 0 | 52 |
| `_with_null` fns | 0 | 0 | 10 |
| public fn names over 50 chars | 109 | 113 | 34 |
| `IntoIterator for &T` | 0 | 0 | 5 |
| `-> Option<JsValue>` | 0 | 0 | 8 |
| `impl BufferSourceLike` | 0 | 0 | 70 |
| `_array_buffer` / `_typed_array` arms | 140 | 140 | 0 |
| `_promise` twins | 277 | 276 | 0 |
| `async fn`s | 277 | 292 | 0 |
| `KVNamespace` get/put/list/delete fns | 70 | 78 | 73 |
| lines | 18,366 | 19,548 | 17,198 |

Every item that is **still worse in B than in A**:
- **`&[String]` → `&[impl JsStringLike]`:** `&[]` needs `&[] as &[&str]`. Literal lists and `&Vec<String>` both work.
- **`Option<&str>` → `Option<impl JsStringLike>` (15):** this remains on raw setters and on required nullable dictionary fields. `None` there needs `None::<&str>`. Builders and expanded unions no longer have it.
- **Generic user code over a helper** must restate a `for<'a> &'a T: IntoWasmAbi` bound when the helper's `T` is unbounded (12 where-clauses, down from phase 1's 66; handle-bounded helpers need none). A needed `T: JsGeneric`.
- **Surface size.** The added items are:
  - `_typed` twins (52), for constructors and nested return-only parameters;
  - `_with_null` arms (10);
  - KV typed reads. With the `_promise` twins gone, `KVNamespace` has 73 get/put/list/delete fns, against 70 in A and 78 in B1 (138 before D1). The longest is `get_with_metadata_text_with_options_typed_js_string`.

  Twins are down to 15, all intended (`text()` and KV values), and every `JsString` mention left is in their signatures.
- **Tail `.await`:** an async member returns `Promise<T>`, which rejects with `JsValue`, so a tail `x.await` in a fn returning `Result<_, Error>` needs `Ok(x.await?)` (37 port sites). A and B1's `async fn`s returned `crate::Error` directly. `x.await?` is unchanged (§7).
- **Surface outside `bindings.rs` (unstable js-sys):** `Reflect::get` → `Result<Option<T>>`, `Array::get` → `Option<T>`, `JSON::stringify` → `Option<String>`, and iterators that rethrow instead of yielding `Result`. Code written against stable js-sys does not compile under the cfg (`worker`, `worker-sys`, serde-wasm-bindgen, wasm-streams).

## 4. Costs

**Size.** Each state is built from its own commit with its own wasm-bindgen pin, on the same
machine. The benchmark worker is a release build run through `wasm-opt -Oz` with
`benchmark/build.sh`'s flags. The test worker is a debug build and covers the whole suite's
surface. gz is `gzip -9`. B was measured at `0bf25a7` (sizes) and `e5e88b9` (times; the commit between only adds `slice = "copy"` to 15 setters). A was re-measured for this report and
reproduces phase 1's numbers exactly. B1 is phase 1's measurement.

| | A | B1 | B | B − A |
|---|---:|---:|---:|---:|
| benchmark `index_bg.wasm` | 120,645 (51,600 gz) | 121,251 (51,735 gz) | 121,231 (51,443 gz) | +586 (+0.5%), -157 gz |
| benchmark `index.js` | 33,871 (6,371 gz) | 35,165 (6,449 gz) | 35,658 (6,358 gz) | +1,787 (+5.3%), -13 gz |
| benchmark wasm imports | 77 | 77 | 80 | +3 |
| test worker glue | 175,754 | 198,024 | 184,903 | +9,149 (+5.2%) |
| test worker wasm imports | 472 | 488 | 490 | +18 |
| test worker generic shims (`// generic import`) | 0 | 177 | 0 | +0 |
| test worker `finally` blocks (owned-string frees) | 10 | 14 | 6 | -4 |

**Compile time** of the `ts-gen-worker` crate from clean (`cargo clean -p ts-gen-worker`, deps
warm, wasm32). The table gives the second of two runs, with A and B measured back to back. B1 is
phase 1's measurement.

| | A | B1 | B | B − A |
|---|---:|---:|---:|---:|
| debug | 14.1 s | 17.6 s | 14.8 s | +0.7 s (+5%) |
| release | 11.1 s | 13.9 s | 11.8 s | +0.8 s (+7%) |
| test crate alone on top (debug) | 4.2 s | 4.3 s | 4.2 s | +0.0 s |

What moved since phase 1 (B1 → B):
- **Generic shims 177 → 0, glue +22,270 B → +9,149 B over A.** J's wire-format grouping makes `&str`, `String`, `&String` and `JsString` one import, and handles one import whether passed by value or by reference. The `// generic import` comments are gone without `--keep-debug`. Owned strings cross borrowed, so the `finally` frees went from 14 to 6 (A has 10).
- **Twins 383 → 15.** `{@string copy}` on small strings means less surface. The output does not depend on it, because unused twins never reach the binary.
- **Compile time +23–25% → +5–7% over A.** D1 removed every `async fn` wrapper and `_promise` twin, and A3 the buffer arm pairs. `bindings.rs` is now 20,733 lines, against A's 22,148, B1's 23,740 and round 1's 23,890.

What B still costs over A:
- **Benchmark: +3 imports, +0.5% wasm (gzip −0.3%).** The import diff, with A's and B's import bodies normalised, is three swaps and three additions:
  - `Error::name` and `Error::message` return a `String` copy instead of a handle (two swaps);
  - `ResponseInit::headers` is read as `Option` (a swap);
  - string bodies cross as UTF-8 (`new Response(getStringFromWasm0(..), init)`) beside the handle form the stream and `null` bodies use;
  - awaiting a `Promise<()>` resolves through a closure typed `fn(())`. js-sys's `IntoFuture` for `Promise<T>` makes one closure cast per awaited wire format;
  - `Promise.reject`, which `catch` on a `Promise<T>` import uses to return a synchronous throw.
- **Test worker: +18 imports, +5.2% glue.** The added imports are distinct wire formats rather than per-type shims:
  - string returns copied into Rust (`toISOString`, `toString`, tuple and record values) where A handed back a `JsString`;
  - string arguments as UTF-8 where A passed a handle (`body`, `prefix`, `type`);
  - byte arguments as wasm-memory views (`put`, `uploadPart`) beside the handle form;
  - awaited wire formats (`()`, `Option<f64>`, `Option<ArrayBuffer>`, …): 5 more closure casts;
  - the port's wider API use: per-type KV reads, `get_with_metadata_typed::<f64>`, generated `[Symbol.iterator]`, `Object::from_iter`/`collect` (`Reflect.set` by string key), three `Function::from_fn` closures.
- **Compile time** in the bindings crate is +5% debug and +7% release, though `bindings.rs` is smaller than A's. That is the mono macro's larger expansion per generic binding. The test crate on top is flat. A second back-to-back pair under more load came out −9% / +1%, so the delta is near this machine's noise.

## 5. "Fleshed out" checklist

Every change B needed, all built (commits and files in `mono-prototype-patches.md`):

| change | where | phase | mono-specific |
|---|---|---|---|
| `js_sys_unstable_apis` for wasm32; serde-wasm-bindgen and wasm-streams patched copies | workers-rs | 1 | no |
| native leaves inside js-sys containers; `&T` generic arguments; awaited `Option<T>` | ts-gen | 1 | yes |
| argument-position `impl Trait` keeps declared generic arity | wasm-bindgen macro | 1 | yes |
| one import per wire format; no `// generic import` comments; `?Sized` mono params; `&Option<T>`, `Option<&T>` args | wasm-bindgen macro + CLI | 2 | yes (mono's own cost) |
| `extends = Parent<A, B>`; `Vec<Option<T>>` returns; blanket `&T` ABI for handles | wasm-bindgen | 2 | partly |
| `catch` on a `Promise<T>` return hands a synchronous throw back as a rejection | wasm-bindgen macro | 2 | no |
| `Map` by `Borrow<Q>`, `Map::get` for every `V` | js-sys | 2 | no |
| `Function<fn(A..) -> R>` with any ABI `A`/`R`, `()` for void; `Function::from_fn` / `once`; typed `Promise::new` | js-sys | 2 | yes |
| sealed `BufferSourceLike` (`&[u8]`, `Vec<u8>`, `ArrayBuffer`, typed arrays, `DataView`) | js-sys | 2 | partly (the handle forms share one wire format) |
| `String` returns from built-ins (`Error`, `Date`, `JSON`, `Intl`); `Date::new_with_value`; `ArrayBuffer` bytes | js-sys | 2 | no |
| iterators yield `T`; `Entries<K, V>`; Map/Set/Object as Rust collections | js-sys | 2 | no |
| stopgaps #1–#5 replaced by ts-gen fixes (`mono-patch` deleted) | ts-gen | 2 | yes |
| `{@string copy\|ref\|both}`; `@retype`; side namespaces | ts-gen + `annotations.d.ts` | 2 | no |
| `_typed` twins (constructors, nested return-only params); concrete construct returns | ts-gen | 2 | yes |
| `T \| null` args: bare arms + `_with_null` arm; bare builder values | ts-gen | 2 | no |
| callbacks with Rust leaves, `Promise<()>` | ts-gen | 2 | yes |
| async members return `Promise<T>`, with no `async fn` / `_promise` pair | ts-gen | 2 | partly |
| `Iterable<T>` → slices; alias leaves; optional `any` → `Option<JsValue>`; generated `IntoIterator` | ts-gen | 2 | no |
| per-literal overload bindings (KV typed reads); literal-only dictionaries set by the binding (`raw_with_column_names`, `get_reader_byob`); shorter overload and inline-type names | ts-gen | 2 | no |
| `ArrayBuffer \| ArrayBufferView` → `impl BufferSourceLike`; `this` returns the receiver; optional `Partial` getters | ts-gen | 2 | partly |
| bare generic-class arguments take any instantiation | ts-gen | 2 | yes |
| `M[K]` over a concrete `M` binds once per key (typed event listeners on subclasses) | ts-gen | 2 | yes |

C would need instead (none built):
- #5167 with the root `&T` slot leak fixed;
- `JsStr<'a>` and a sealed `JsStringLike::as_js_str`, plus a list either type;
- tagged enums with `From` impls for the 12 primitive unions;
- hand-written `IntoJsGeneric`/`FromJsGeneric` for every user type used as a type argument.

Every "no" row carries over to C as is. The js-sys collection behaviour would carry over too,
along with the same ecosystem break.

## 6. Remaining gaps

Not built, by owner. Every audit item not listed here is done.

**Ecosystem (workers-rs).**
- `worker` and `worker-sys` are not ported to `js_sys_unstable_apis`, so the repo's 145-test `chomp test` stays excluded. B's js-sys changes behaviour, not just types:
  - iterators rethrow instead of yielding `Result`;
  - `Reflect::get` / `Array::get` return `Option`;
  - `JSON::stringify` returns `Option<String>`;
  - `Map::get` borrows its key.

  Shipping B means porting `worker` (and depending on patched serde-wasm-bindgen and wasm-streams), or stabilising these js-sys APIs first.
- Every patch is prototype quality. `mono-prototype-patches.md` groups them by upstream PR.

**ts-gen.**
- `&[u8]` into a transferring callee throws (audit B1): `ReadableByteStreamController::enqueue` and `ReadableStreamBYOBRequest::respond_with_new_view` take `impl BufferSourceLike` without `slice = "copy"`, and workerd requires a detachable buffer. More broadly, `&[u8]` is a view unless a site opts into the copy, though the view is proven safe only at the callees checked here.
- `[A, ...B[]]` lowers to a fixed 2-tuple (audit B2). D1 `raw`'s `{ columnNames: true }` arm returns `[string[], ...T[]]`, but `ArrayTuple<(Array<String>, Array<T>)>` reads the first row as `Array<T>`, and the later rows are unreachable. A and B1 had the same lowering.
- Per-key listeners have only the plain and listener-object arms (audit B4). `{ once: true }` or `{ signal }` still need the erased `add_event_listener_with_options` and a `fn(JsValue)` handler; the same holds for `remove_*`.
- `IncomingRequestCfProperties` has numbered builders, from crossing a dictionary-union field with the type's other unions. Only mocks build an incoming `cf`.
- `File | string` and other handle-or-string unions stay `JsValue`. That needs a ts-gen enum in any mode.

**wasm-bindgen / js-sys.**
- `none = "null"` (exact `null` for an omitted argument, for non-WebIDL libraries) is deferred. `JsNullable<T>` is still the way to send it.
- Array callback methods other than `for_each` (`map`, `filter`, …) stay erased. Rust iteration covers them.
- `Option<JsValue>` treats only `undefined` as `None`, so `M | null` is `None` for `null` at `M = f64` but `Some(null)` at `M = JsValue`. The port filters by hand (`src/ext/kv.rs:133`). One `Option<JsValue>` cannot mean both `T | undefined` and `T | null`, so this needs design.

**By design (no action).**
- `&[] as &[&str]`. `None::<&str>` on raw setters and required nullable fields (15 arguments).
- `any` values stay `JsValue`.
- `void | Promise<void>` callback returns are `Option<Promise<()>>`. That is correct, but a hand-built handler closure returns `None`/`Some(p)`; the macros hide it.
- Mono's compile-time cost (§4).

## 7. Decisions taken during the prototype (reversible, for review)

Policies chosen while building B to finish a port idiom, flagged for review. Each can be reverted in
ts-gen alone.

- **Return-only type parameters (audit #8).** A method whose return is exactly `T` or `T | null` keeps the
  generic, so the turbofish is the cast (`storage.get::<f64>(k)`, like `parse::<T>()`). When the parameter is
  nested in the return (`getWithMetadata<Metadata>`, `list<Metadata>`), the default form erases it and a
  `_typed::<T>` twin binds it, the same suffix constructors use. Keeping the generic everywhere was rejected:
  Rust fns have no default type parameters, so `kv.list().await?` would need a turbofish with metadata unused.
- **Async members return `Promise<T>` (D1).** Each async member is one binding returning `Promise<T>`, with `T`
  the awaited form (`Promise<Option<String>>`, `Promise<()>`). There is no `async fn` / `_promise` pair: the 293
  `_promise` twins are gone and public fns went from 3,628 to 3,335. js-sys's `Promise<T>` is
  `IntoFuture<Output = Result<T, JsValue>>`, so `x.await?` is unchanged, and the same value passes to JS
  (`block_concurrency_while(&callback)` returns the promise that holds the gate). The imports keep `catch`: on a
  `Promise<T>` return, wasm-bindgen hands a synchronous throw back as a rejected promise, so it surfaces at
  `.await`. The cost is the tail position. A tail `x.await` in a fn returning `Result<_, Error>` becomes
  `Ok(x.await?)`, because the promise rejects with `JsValue` (37 port sites).

  There are two alternatives:
  - A ts-gen-worker wrapper with `Output = Result<T, Error>` and an `into_promise()`. It keeps tail sites
    unchanged, but it adds a type to every async return.
  - `Promise<T, E = JsValue>` in js-sys, with `IntoFuture<Output = Result<T, E>> where E: From<JsValue>` and `E`
    erased for the ABI (audit B5). ts-gen would emit `Promise<T, crate::Error>`, so the tails and the configured
    error type come back, with no wrapper and no change for existing `Promise<T>` users. The cost is a second
    parameter on `Promise`.
- **Typed event listeners per key (D2).** `addEventListener<K extends keyof M>(type: K, handler: (e: M[K]) => …)`
  on a class whose `M` is a declared interface gets one binding per key, with the type string set by the binding:
  `WebSocket::add_event_listener_message(&Function<fn(MessageEvent)>)`. Subclasses declare them for their first
  parent's methods, and the generic string form stays. Overloads told apart by one literal are now named by it.
  The alternative was an `on_message(impl FnMut(MessageEvent))` form that owns the closure. That is shorter at the
  call site but hides the `Function` that `removeEventListener` needs.
- **`T | null` arguments are `Option<…>`, and `None` crosses as `undefined`.** This conflicts with #51's
  no-coercion rule. Expanded unions keep an exact `null` through their `_with_null` arms.
- **An optional parameter's trailing `null` arm is dropped.** Omitting the argument covers it. That is exact
  for WebIDL `T? = null`; a non-WebIDL callee that tells missing from `null` loses the bare-`null` call.
- **Optional properties typed only `never` or `undefined` have no accessors.** This removes
  `D1Result::error()`, which `worker` exposes as `Option<String>`.

