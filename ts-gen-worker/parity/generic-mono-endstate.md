# Generic mono end state: the built prototype (B) vs today (A) vs a non-mono end state (C)

A = non-mono today (`a4cbe97` bindings, `104579e` port). B = the phase-2 prototype: mono +
`js_sys_unstable_apis` + js-sys made fully mono + the ts-gen and wasm-bindgen fixes, built on
`proto/mono` in workers-rs-3 (ts-gen and wasm-bindgen `proto/mono`, all local; every patch is in
`mono-prototype-patches.md`). B1 = phase 1 (`2aa55cc` port), kept as a column where it shows what
phase 2 added. C = non-mono + wasm-bindgen#5167 + a `JsStr<'a>` either type behind
`impl JsStringLike` + tagged enums for primitive unions: **design, not built**, written by hand
from the #5167 compile matrix (`~/Projects/wasm-bindgen-5` `2497681c1`, stable js-sys).

## Verdict

1. B builds as generated, with no stopgaps, and passes the suite (194/194, plain and `-http`). The port has none of: `JsString`, `String::from(<js>)`, `JsNullable`, `None::<…>`, `map_err(|e| e.to_string())`, `JsFuture::from` at call sites. `unchecked_*` casts went from 29 (A) to 6. Each of the 6 sits at an `any`-typed or erased boundary (§2).
2. Everything phase 1 assumed is built, and the R audit's items are done except the ones in §6. Phase 2 also built Rust iteration, `Object<T>` as a map, typed KV reads, byte-stream bodies and shorter overload names.
3. The cost in commits: 11 wasm-bindgen/js-sys commits and 19 ts-gen commits on top of phase 1. Under the cfg, js-sys is a different API in which collections behave like Rust collections. That API break, and the fallout in `worker`, serde-wasm-bindgen and wasm-streams, is the price of the containers. `worker` is not ported (§6).
4. The cost in size and time: benchmark wasm +1,380 (+1.1%), test glue +7,455 (+4.2%) (phase 1: +12.7%), test imports +15 (phase 1: +16), 0 generic shims (phase 1: 177). `ts-gen-worker` compiles +23% debug and +25% release (§4).
5. Mono's own lead over C is unchanged from phase 1:
   - leaf type arguments on generated generics (`get::<f64>`, `get_with_metadata_typed::<f64>`, `first`);
   - primitive unions as one function;
   - callbacks with Rust leaves.

   Most other gains are ts-gen lowering or unstable js-sys, which a non-mono build could also adopt (§5).

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
Response::new_with_array_buffer_and_init(&ArrayBuffer::from(bytes), &init)?
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
self.kv.get_text_with_options(&self.key, &options).await                      // sets type: "text"; Option<String>
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
// B: test/src/r2.rs:74-79, 105, 154, 359
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
// B: test/src/d1.rs:16-19, 57-61, 261
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
let on_message = Function::from_fn(move |event: JsValue| { /* event.unchecked_into() */ });
ws.add_event_listener("message", &on_message)?;                              // &Function<fn(JsValue)>
event.data().as_string()
```
```rust
// C: B's send/close call sites; callbacks stay erased (`Function<fn(JsValue) -> Undefined>`).
```
- `&str`, `String` and `&String` share one import, borrowed in JS and dropped in Rust after the call. Phase 1 had one import per Rust type.
- Callbacks take Rust leaves, with `void` as `()`, so a unit closure fits `Function::from_fn`.
- The event is still `JsValue`. `addEventListener<K>`'s `(e: Map[K]) => …` is an indexed access type, so one cast per listener stays.

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
// B: test/src/send_email.rs:136; content_id is signature only
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
// B: src/ext/fetch.rs:26, src/ext/delay.rs:54, src/ext/kv.rs:160, test/src/kv.rs:79, test/src/r2.rs:184, 354
fetch_with_init(&req, &init)                                               // the primary arm (Request) is implied
set_timeout_with_ms_delay(&callback, ms)
kv.get_text_with_options(&key, &options)                                   // one binding per `type` literal
kv.put_with_options(key, value, &options)                                  // a dictionary arm is named by its parameter
bucket.get_with_options(key, &options)
bucket.put_with_str_and_options(key, value, &options)                      // inline `onlyIf` form: put_with_str_and_r2_bucket_put_options
```
Public names over 50 characters fell from 109 to 54. The longest left are `_with_slice_and_options`
KV reads and `IncomingRequestCfProperties`' numbered builders. Every rule here is ts-gen naming,
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
| `unchecked_into` / `unchecked_ref` | 29 | 19 | 6 |
| `dyn_into` / `dyn_ref` | 7 | 7 | 6 |
| turbofish `::<JsValue` | 9 | 7 | 3 |
| all turbofish `::<` | 92 | 91 | 86 |
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
| `Closure::` | 4 | 4 | 2 |
| `._0()` / `._1()` | 10 | 10 | 2 |
| `Uint8Array::new(&…)` re-wraps | 5 | 5 | 0 |
| lines | 7,444 | 7,386 | 7,222 |

The 6 remaining casts:
- the wasm-streams boundary (`web_sys::ReadableStream` → `ReadableStream<Uint8Array>`, `src/ext/streams.rs:164`);
- `EnvExt::get`, which is unchecked on purpose;
- `HeadersInit` read back from a `ResponseInit` (`src/ext/response.rs:37`);
- a serialized struct passed as an entries object (`src/ext/serde.rs:64`);
- the two event-map listeners (`test/src/util.rs:34,38`).

The `.as_string()` / `dyn_*` calls are on `any` values: `MessageEvent::data`, attachments,
FormData `File | string`, SQL values and `Error::cause`. The one `::<JsValue>` left on a
binding is `block_concurrency_while_promise` (§6).

## 3. API diff

`tools/api-diff` over the generated `bindings.rs` (item-keyed signatures):

- **B vs A**: 1512 items changed, 278 removed, 491 added, 3265 unchanged. Most removed/added pairs are renames (overload suffixes, `_with_null` arms, KV typed reads).
- **B vs B1**: 356 changed, 639 removed (376 of them `_js_string` twins), 481 added, 4431 unchanged.

Type-level counts (doc and comment lines skipped):

| pattern | A `a4cbe97` | B1 `2aa55cc` | B |
|---|---:|---:|---:|
| `JsString` (word) | 175 | 422 | 64 |
| `_js_string` twins | 0 | 383 | 15 |
| `Map<JsString, …>` | 44 | 0 | 0 |
| `Array<JsString>` | 3 | 18 | 0 |
| `Promise<JsString>` | 8 | 0 | 0 |
| `Object<JsString>` | 28 | 0 | 0 |
| `Iterator<JsString>` | 6 | 0 | 0 |
| `JsNullable<JsString>` | 39 | 24 | 0 |
| `JsNullable<…>` | 136 | 91 | 0 |
| `JsOption<…>` | 110 | 95 | 0 |
| `Function<fn(…JsString…)>` | 8 | 8 | 0 |
| `Function<fn(…)>` | 101 | 101 | 113 |
| `-> Undefined` | 17 | 17 | 0 |
| `Promise<Undefined>` | 163 | 162 | 0 |
| `Promise<()>` | 0 | 0 | 162 |
| `Boolean` / `Number` | 32 | 8 | 0 |
| `impl JsStringLike` | 0 | 1,268 | 1,436 |
| `Option<impl JsStringLike>` | 0 | 7 | 16 |
| `&[impl JsStringLike]` | 0 | 117 | 165 |
| `impl js_sys::Js…Like` (primitive unions) | 0 | 13 | 13 |
| `Option<&str>` / `&[String]` | 124 | 0 | 0 |
| `: JsGeneric` bounds | 666 | 75 | 0 |
| `for<'a>` where-clauses | 0 | 66 | 12 |
| `_typed` fns | 0 | 0 | 53 |
| `_with_null` fns | 0 | 0 | 16 |
| public fn names over 50 chars | 109 | 113 | 54 |
| `IntoIterator for &T` | 0 | 0 | 5 |
| `-> Option<JsValue>` | 0 | 0 | 8 |
| lines | 18,366 | 19,548 | 20,126 |

Every item that is **still worse in B than in A**:
- **`&[String]` → `&[impl JsStringLike]`:** `&[]` needs `&[] as &[&str]`. Literal lists and `&Vec<String>` both work.
- **`Option<&str>` → `Option<impl JsStringLike>` (16):** this remains on raw setters and on required nullable dictionary fields. `None` there needs `None::<&str>`. Builders and expanded unions no longer have it.
- **Generic user code over a helper** must restate a `for<'a> &'a T: IntoWasmAbi` bound when the helper's `T` is unbounded (12 where-clauses, down from phase 1's 66; handle-bounded helpers need none). A needed `T: JsGeneric`.
- **Surface size.** The added items are:
  - `_typed` twins (53), for constructors and nested return-only parameters;
  - `_with_null` arms (16);
  - KV typed reads.

  Twins are down to 15, all intended (`text()` and KV values). The 64 `JsString` mentions left are all inside those twin families.
- **Surface outside `bindings.rs` (unstable js-sys):** `Reflect::get` → `Result<Option<T>>`, `Array::get` → `Option<T>`, `JSON::stringify` → `Option<String>`, and iterators that rethrow instead of yielding `Result`. Code written against stable js-sys does not compile under the cfg (`worker`, `worker-sys`, serde-wasm-bindgen, wasm-streams).

## 4. Costs

**Size.** Each state is built from its own commit with its own wasm-bindgen pin, on the same
machine. The benchmark worker is a release build run through `wasm-opt -Oz` with
`benchmark/build.sh`'s flags. The test worker is a debug build and covers the whole suite's
surface. gz is `gzip -9`. A was re-measured for this report and reproduces phase 1's numbers
exactly. B1 is phase 1's measurement.

| | A | B1 | B | B − A |
|---|---:|---:|---:|---:|
| benchmark `index_bg.wasm` | 120,645 (51,600 gz) | 121,251 (51,735 gz) | 122,025 (52,005 gz) | +1,380 (+1.1%), +405 gz |
| benchmark `index.js` | 33,871 (6,371 gz) | 35,165 (6,449 gz) | 34,745 (6,296 gz) | +874 (+2.6%), -75 gz |
| benchmark wasm imports | 77 | 77 | 78 | +1 |
| test worker glue | 175,754 | 198,024 | 183,209 | +7,455 (+4.2%) |
| test worker wasm imports | 472 | 488 | 487 | +15 |
| test worker generic shims (`// generic import`) | 0 | 177 | 0 | +0 |
| test worker `finally` blocks (owned-string frees) | 10 | 14 | 6 | -4 |

**Compile time** of the `ts-gen-worker` crate from clean (`cargo clean -p ts-gen-worker`, deps
warm, wasm32). The table gives the second of two runs, with A and B measured back to back. B1 is
phase 1's measurement.

| | A | B1 | B | B − A |
|---|---:|---:|---:|---:|
| debug | 14.3 s | 17.6 s | 17.6 s | +3.3 s (+23%) |
| release | 11.2 s | 13.9 s | 14.0 s | +2.8 s (+25%) |
| test crate alone on top (debug) | 4.1 s | 4.3 s | 4.2 s | +0.0 s |

What moved since phase 1 (B1 → B):
- **Generic shims 177 → 0, glue +22,270 B → +7,455 B over A.** J's wire-format grouping makes `&str`, `String`, `&String` and `JsString` one import, and handles one import whether passed by value or by reference. The `// generic import` comments are gone without `--keep-debug`. Owned strings cross borrowed, so the `finally` frees went from 14 to 6 (A has 10).
- **Twins 383 → 15.** `{@string copy}` on small strings means less surface. The output does not depend on it, because unused twins never reach the binary.

What B still costs over A:
- **Benchmark: +1 import, +1.1% wasm.** The import diff, with A's and B's import bodies normalised, is three swaps and one addition:
  - `Error::name` and `Error::message` return a `String` copy instead of a handle (two swaps);
  - `ResponseInit::headers` is read as `Option` (a swap);
  - string bodies cross as UTF-8 (`new Response(getStringFromWasm0(..), init)`) beside the handle form the stream and `null` bodies use (the one addition).
- **Test worker: +15 imports, +4.2% glue.** The added imports are distinct wire formats rather than per-type shims:
  - string returns copied into Rust (`toISOString`, `toString`, tuple and record values) where A handed back a `JsString`;
  - string arguments as UTF-8 where A passed a handle (`body`, `prefix`);
  - the port's wider API use: per-type KV reads, `get_with_metadata_typed::<f64>`, generated `[Symbol.iterator]`, `Object::from_iter`/`collect` (`Reflect.set` by string key), three `Function::from_fn` closures.
- **Compile time** sits in the bindings crate. The mono macro emits more code per generic binding than the erased path, and the surface grew (`_typed` twins, `_with_null` arms, KV typed reads). Phase 1 put 1.0 s on the twins; their removal is offset by the new items, so B lands on B1's time. The test crate on top is flat, so instantiation at use sites costs little.

## 5. "Fleshed out" checklist

Every change B needed, all built (commits and files in `mono-prototype-patches.md`):

| change | where | phase | mono-specific |
|---|---|---|---|
| `js_sys_unstable_apis` for wasm32; serde-wasm-bindgen and wasm-streams patched copies | workers-rs | 1 | no |
| native leaves inside js-sys containers; `&T` generic arguments; awaited `Option<T>` | ts-gen | 1 | yes |
| argument-position `impl Trait` keeps declared generic arity | wasm-bindgen macro | 1 | yes |
| one import per wire format; no `// generic import` comments; `?Sized` mono params; `&Option<T>`, `Option<&T>` args | wasm-bindgen macro + CLI | 2 | yes (mono's own cost) |
| `extends = Parent<A, B>`; `Vec<Option<T>>` returns; blanket `&T` ABI for handles | wasm-bindgen | 2 | partly |
| `Map` by `Borrow<Q>`, `Map::get` for every `V` | js-sys | 2 | no |
| `Function<fn(A..) -> R>` with any ABI `A`/`R`, `()` for void; `Function::from_fn`; typed `Promise::new` | js-sys | 2 | yes |
| `String` returns from built-ins (`Error`, `Date`, `JSON`, `Intl`); `Date::new_with_value`; `ArrayBuffer` bytes | js-sys | 2 | no |
| iterators yield `T`; `Entries<K, V>`; Map/Set/Object as Rust collections | js-sys | 2 | no |
| stopgaps #1–#5 replaced by ts-gen fixes (`mono-patch` deleted) | ts-gen | 2 | yes |
| `{@string copy\|ref\|both}`; `@retype`; side namespaces | ts-gen + `annotations.d.ts` | 2 | no |
| `_typed` twins (constructors, nested return-only params); concrete construct returns | ts-gen | 2 | yes |
| `T \| null` args: bare arms + `_with_null` arm; bare builder values | ts-gen | 2 | no |
| callbacks with Rust leaves, `Promise<()>` | ts-gen | 2 | yes |
| `Iterable<T>` → slices; alias leaves; optional `any` → `Option<JsValue>`; generated `IntoIterator` | ts-gen | 2 | no |
| per-literal overload bindings (KV typed reads); shorter overload and inline-type names | ts-gen | 2 | no |
| bare generic-class arguments take any instantiation | ts-gen | 2 | yes |

C would need instead (none built):
- #5167 with the root `&T` slot leak fixed;
- `JsStr<'a>` and a sealed `JsStringLike::as_js_str`, plus a list either type;
- tagged enums with `From` impls for the 13 primitive unions;
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
- Names: `json_with_maybe_init` keeps upstream's parameter name, and `IncomingRequestCfProperties` has numbered builders for a union of dictionaries.
- `File | string` and other handle-or-string unions stay `JsValue`. That needs a ts-gen enum in any mode.

**wasm-bindgen / js-sys.**
- `none = "null"` (exact `null` for an omitted argument, for non-WebIDL libraries) is deferred. `JsNullable<T>` is still the way to send it.
- A `Closure::once` does not infer a typed `Function`'s parameters through `from_closure`, so `block_concurrency_while_promise::<JsValue>` keeps its turbofish (`test/src/util.rs:85`). An `FnOnce` form of `Function::from_fn` would fix it.
- Array callback methods other than `for_each` (`map`, `filter`, …) stay erased. Rust iteration covers them.

**By design (no action).**
- `&[] as &[&str]`. `None::<&str>` on raw setters and required nullable fields (16 arguments).
- Event-map listeners stay `fn(JsValue)` because `addEventListener<K>` indexes a map type. `any` values stay `JsValue`.
- `void | Promise<void>` callback returns are `Option<Promise<()>>`. That is correct, but a hand-built handler closure returns `None`/`Some(p)`; the macros hide it.
- Mono's compile-time cost (§4).
