# Generic mono end state: the built prototype (B) vs today (A) vs a non-mono end state (C)

A = the non-mono `ts-gen-worker` today (bindings at `a4cbe97`, port at `104579e`). B = this branch:
mono generics + `js_sys_unstable_apis` + js-sys made fully mono + the ts-gen and wasm-bindgen fixes,
with ts-gen and wasm-bindgen pinned as submodules at their `proto/mono` tips. Every patch is listed
in `mono-prototype-patches.md`. B1 = phase 1, an earlier mono build (`2aa55cc` port), kept as a column
where it shows what phase 2 (B) added. C = non-mono + wasm-bindgen#5167 + a `JsStr<'a>` either type
behind `impl JsStringLike` + tagged enums for primitive unions: **design, not built**, written by hand
from the wasm-bindgen#5167 compile matrix (wasm-bindgen `2497681c1`, stable js-sys).

## Verdict

1. B builds as generated, with no hand patches to the bindings, and passes the suite (195/195, plain and `-http`; A has 194, plus the `MessageEvent` catch-all test). The port has none of: `JsString`, `String::from(<js>)`, `JsNullable::wrap`/`new`, `None::<…>`, `.set_x(Some(…))`, `map_err(|e| e.to_string())`, `JsFuture::from` at call sites. `unchecked_*` casts went from 29 (A) to 4. Each of the 4 sits at an `any`-typed or erased boundary (§2).
2. Everything phase 1 assumed is built, and every review finding is done except the ones in §6. Phase 2 also built Rust iteration, `Object<T>` as a map, typed KV reads, byte-stream bodies, one argument for buffer sources, one `Promise<T, crate::Error>` per async method, shorter overload names, a null model (writes keep `null` exact, reads fold it into `Option`, §7), and signatures whose bounds are the declaration's (no `WasmDescribe` or `for<'__wbg>` in rustdoc, §3).
3. The cost in commits: 21 wasm-bindgen/js-sys commits (three are wasm-bindgen#5385) and 38 ts-gen commits on top of phase 1. Under the cfg, js-sys is a different API in which collections behave like Rust collections. That API break, and the fallout in `worker`, serde-wasm-bindgen and wasm-streams, is the price of the containers. `worker` is not ported (§6).
4. The cost in size and time: benchmark wasm +633 (+0.5%), test glue +9,369 (+5.3%) (phase 1: +12.7%), test imports +20 (phase 1: +16), 0 generic shims (phase 1: 177). `ts-gen-worker` compiles +5% debug and +6% release (§4).
5. Mono's own lead over C is unchanged from phase 1:
   - leaf type arguments on generated generics (`get::<f64>`, `get_with_metadata_typed::<f64>`, `first`);
   - primitive unions as one function;
   - callbacks with Rust leaves.

   Buffer-source unions are only partly a mono lead: the handle forms share one wire format, so C could take them
   through an inline wrapper over `&JsValue`. Only the `&[u8]` view and the `Vec<u8>` copy need per-instantiation
   lowering.

   Most other gains are ts-gen lowering or unstable js-sys, which a non-mono build could also adopt (§5).
6. Lowering policies chosen while building B, each reversible in ts-gen or `annotations.d.ts`, are listed in §7 for review.

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
pub struct HeadersIter(IntoIter<ArrayTuple<(String, String)>>);         // String: JsGeneric (wasm-bindgen#5167)
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
- B drops the nullable wrapper on every builder value. Each nullable field also has a `null` form (`signal_with_null()`, `body_with_null()`).
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
  - `&[u8]`, the form the port uses (`&bytes[..]`), is copied into a fresh `Uint8Array` by default. That is always safe, including for the 15 dictionary setters that store their value (`R2PutOptions::md5`, crypto `iv`/`salt`, `RequestInit` body, attachment content). `test/src/r2.rs:349-356` zeroes the digest after `.md5(&hash[..])`, and R2 still verifies it.
- `{@slice view}` in `annotations.d.ts` makes a `&[u8]` argument a view into wasm memory, valid only for the call. It is set only on callees that copy or read the bytes before returning: `Response`'s body (Fetch spec), KV and R2 `put` and `uploadPart` (workerd copies), and WebCrypto, which works synchronously (29 view arguments, 16 copies).
- `{@slice handle}` takes buffer handles only, for `ReadableStreamBYOBRequest::respond_with_new_view`, which must hand back the request's own buffer. `ReadableByteStreamController::enqueue` transfers its chunk, so it keeps the copy.
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
// B: src/ext/kv.rs:113-120, 157-174, test/src/kv.rs:88-90, 162-176, 112
let options = KVNamespaceGetOptionsPartial::builder().cache_ttl(ttl).build();  // Partial<X>: all fields optional
self.kv.get_text_with_options(&self.key, &options).await                      // sets type: "text"; Option<String>
self.kv.get_array_buffer_with_options(&self.key, &options).await?.map(Vec::from)
let result = store.get_with_metadata_typed::<f64>("put_b").await?;           // nested return-only param
kv_assert_eq!(result.metadata(), Some(100.0))?;
let values = store.get_with_slice(&["bulk_a", "bulk_b", "bulk_missing"]).await?; // Map<String, JsNullable<String>>
let text = |key: &str| values.get(key).and_then(|v| v.into_option());           // Borrow<str> key
kv_assert_eq!(text("bulk_a"), Some("value_a".to_string()))?;
kv_assert_eq!(values.has("bulk_missing"), true)?;                               // present, value null
let values = store.get_with_slice(&[] as &[&str]).await?;                        // element type needed
KVNamespaceListOptions::builder().prefix("put_c")                               // .prefix_with_null() sends null
// KVNamespaceListResult, KVNamespaceListKey: output-only, so getters only (no new, setters, builder)
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
- The bulk result is a `Map<String, JsNullable<String>>`: a nullable type argument keeps `JsNullable<T>`, now with the native leaf. `get` takes a borrowed key and returns `Option<V>`, so a missing key is `None` and a `null` value is `Some(v)` with `v.is_null()`. `.and_then(|v| v.into_option())` folds both.
- `KVNamespaceListResult`, `KVNamespaceListKey` and `KVNamespaceGetWithMetadataResult` are only ever returned, so they have no constructor, setters or builder. A and B1 had a write side for all three.
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
// B: test/src/counter.rs:66, test/src/container.rs:128, test/src/util.rs:31-41, 57-61
ws.send_with_str(format!("{count}"))?;                                       // impl JsStringLike
dst.close_with_code_and_reason(reason.code(), reason.reason())?;
let on_message = Function::from_fn(move |event: MessageEvent| { .. });
ws.add_event_listener_message(&on_message)?;                                 // &Function<fn(MessageEvent)>
match event.data() { DataKind::String(text) => Some(text), _ => None }       // @retype: string | ArrayBuffer | Blob | any
```
```rust
// C: B's send/close call sites; callbacks stay erased (`Function<fn(JsValue) -> Undefined>`).
```
- `&str`, `String` and `&String` share one import, borrowed in JS and dropped in Rust after the call. Phase 1 had one import per Rust type.
- Callbacks take Rust leaves, with `void` as `()`, so a unit closure fits `Function::from_fn`.
- `addEventListener<K extends keyof M>(type: K, handler: (e: M[K]) => …)` on `EventTarget<WebSocketEventMap>` binds once per key on the concrete subclass: `add_event_listener_message(&Function<fn(MessageEvent)>)`, `add_event_listener_close(..)`. The generic string form stays beside them, so the casts are gone (§7).

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
field's value bare, so there is no `None` to spell. `contentId` is optional (`undefined`), not
nullable, so the raw setter keeps `Option` and `None` there still needs `None::<&str>` (§3).

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
// B: src/ext/fetch.rs:26, src/ext/delay.rs:54, src/ext/kv.rs:159, test/src/kv.rs:79, test/src/r2.rs:184, 358
fetch_with_init(&req, &init)                                               // the primary arm (Request) is implied
set_timeout_with_ms_delay(&callback, ms)
kv.get_text_with_options(&key, &options)                                   // one binding per `type` literal
kv.put_with_options(key, value, &options)                                  // a dictionary arm is named by its parameter
bucket.get_with_options(key, &options)
bucket.put_with_str_and_options(key, value, &options)                      // inline `onlyIf` form: put_with_str_and_r2_bucket_put_options
```
Public names over 50 characters fell from 109 to 50. Most of the rest are the per-key
listener arms (`remove_event_listener_message_with_event_listener_object_and_options`),
`IncomingRequestCfProperties`' numbered builders and spelled-out multi-parameter arms. Every rule here is ts-gen naming,
so C could adopt it too.

### Env vars and secrets

```rust
// A: test/src/alarm.rs:50-51, test/src/form.rs:139-140
let namespace: DurableObjectNamespace = js_sys::Reflect::get(&env, &"ALARM".into())?.unchecked_into();
let val = js_sys::Reflect::get(&env, &name.into())?;
val.as_string().unwrap_or_default()
```
```rust
// B: test/src/alarm.rs:50, test/src/form.rs:146, test/types/env.d.ts
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
// A: test/src/form.rs:17-22, 52
fn from_js(value: JsValue) -> Option<FormEntry> {                            // get: File | string | null -> JsValue
    if let Some(s) = value.as_string() { Some(FormEntry::Field(s)) }
    else { value.dyn_into::<File>().ok().map(FormEntry::File) }
}
FormEntry::from_js(form.get(NAME))
```
```rust
// B: test/src/form.rs:16-23, 61
match value { GetReturnKind::String(s) => Self::Field(s), GetReturnKind::File(f) => Self::File(f) }
form.get(NAME).map(FormEntry::from)                                         // Option<GetReturnKind>
```
A nullable handle-or-string union is `Option` of a generated enum (§7). That is ts-gen
lowering, so C could adopt it. `get_all()` and `for (name, value) in &form` still yield
`JsValue`, because a `Vec<T>` needs a handle `T`.

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
| `.as_string()` | 14 | 9 | 6 |
| `unchecked_into` / `unchecked_ref` | 29 | 19 | 4 |
| `dyn_into` / `dyn_ref` | 7 | 7 | 6 |
| turbofish `::<JsValue` | 9 | 7 | 1 |
| all turbofish `::<` | 92 | 91 | 84 |
| `let x: T =` annotations | 85 | 80 | 75 |
| `None::<…>` | 0 | 0 | 0 |
| `&[] as &[…]` | 0 | 1 | 1 |
| `JsValue::from` / `from_str` / `from_f64` | 29 | 22 | 22 |
| `JsNullable::wrap` / `new` | 44 | 44 | 0 |
| `JsNullable` (any mention) | 57 | 57 | 2 |
| `.set_x(Some(…))` | 0 | 0 | 0 |
| `.is_null()` | 5 | 5 | 3 |
| `Promise::<…>` | 0 | 1 | 0 |
| `Reflect::` | 13 | 13 | 3 |
| `map_err(\|e\| e.to_string())` | 21 | 21 | 0 |
| `console.with(…)` | 19 | 19 | 7 |
| `JsFuture::from` | 6 | 6 | 1 |
| `Closure::` | 4 | 4 | 1 |
| `._0()` / `._1()` | 10 | 10 | 2 |
| `Uint8Array::new(&…)` re-wraps | 5 | 5 | 0 |
| lines | 7,444 | 7,386 | 7,233 |

The 4 remaining casts:
- the wasm-streams boundary (`web_sys::ReadableStream` → `ReadableStream<Uint8Array>`, `src/ext/streams.rs:161`);
- `EnvExt::get`, which is unchecked on purpose (`src/ext/env.rs:42`);
- `HeadersInit` read back from a `ResponseInit` (`src/ext/response.rs:37`);
- a serialized struct passed as an entries object (`src/ext/serde.rs:61`).

The `.as_string()` / `dyn_*` calls are on `any` values: attachments, `FormData::get_all`'s
entries, SQL values and `Error::cause`. No `::<JsValue>` is left on a binding:
`Function::once` gives `block_concurrency_while` its `T` from the closure's return.

## 3. API diff

`tools/api-diff` over the generated `bindings.rs` (item-keyed signatures):

- **B vs A**: 1273 items changed, 991 removed, 456 added, 2791 unchanged. Most removed/added pairs are renames (overload suffixes, `_with_null` arms, KV typed reads).
- **B vs B1**: 506 changed, 1352 removed (376 of them `_js_string` twins), 446 added, 3568 unchanged.

Type-level counts (doc and comment lines skipped):

| pattern | A `a4cbe97` | B1 `2aa55cc` | B |
|---|---:|---:|---:|
| `JsString` (word) | 175 | 422 | 21 |
| `_js_string` twins | 0 | 383 | 15 |
| `Map<JsString, …>` | 44 | 0 | 0 |
| `Array<JsString>` | 3 | 18 | 0 |
| `Promise<JsString>` | 8 | 0 | 0 |
| `Promise<JsString, crate::Error>` | 0 | 0 | 3 |
| `Object<JsString>` | 28 | 0 | 0 |
| `Iterator<JsString>` | 6 | 0 | 0 |
| `JsNullable<JsString>` | 39 | 24 | 0 |
| `JsNullable<…>` | 136 | 91 | 10 |
| `JsOption<…>` | 110 | 95 | 0 |
| `Function<fn(…JsString…)>` | 8 | 8 | 0 |
| `Function<fn(…)>` | 101 | 101 | 159 |
| `-> Undefined` | 17 | 17 | 0 |
| `Promise<Undefined>` | 163 | 162 | 0 |
| `Promise<()>` (callbacks, getters) | 0 | 0 | 100 |
| `Promise<(), crate::Error>` (methods) | 0 | 0 | 60 |
| `Boolean` / `Number` | 32 | 8 | 0 |
| `impl JsStringLike` | 0 | 1,268 | 1,081 |
| `Option<impl JsStringLike>` | 0 | 7 | 4 |
| `Option<&…>` arguments | 10 | 3 | 2 |
| setters | 705 | 701 | 567 |
| dictionary builders | 110 | 110 | 91 |
| `&[impl JsStringLike]` | 0 | 117 | 82 |
| `impl js_sys::Js…Like` (primitive unions) | 0 | 13 | 12 |
| `Option<&str>` / `&[String]` | 124 | 0 | 0 |
| `: JsGeneric` bounds | 666 | 75 | 0 |
| `for<'a>` where-clauses | 0 | 66 | 0 |
| `_typed` fns | 0 | 0 | 51 |
| `_with_null` fns (public) | 0 | 0 | 17 |
| public fn names over 50 chars | 109 | 113 | 50 |
| `IntoIterator for &T` | 0 | 0 | 5 |
| `-> Option<JsValue>` | 0 | 0 | 12 |
| `impl BufferSourceLike` | 0 | 0 | 67 |
| `_array_buffer` / `_typed_array` arms | 140 | 140 | 0 |
| `_promise` twins | 277 | 276 | 0 |
| `async fn`s | 277 | 292 | 0 |
| `KVNamespace` get/put/list/delete fns | 70 | 78 | 73 |
| lines | 18,366 | 19,548 | 16,348 |

**Rustdoc signatures.** With wasm-bindgen#5385 the mono macro adds no ABI bounds to the public wrapper, so
the declaration's bounds are the whole contract, and ts-gen states exactly those (`1fb83ab`):
- `impl JsStringLike` / `impl BufferSourceLike` arguments and concrete imported types (`&RequestInit<Cf>`,
  `Promise<T, crate::Error>`, `&Function<fn(MessageEvent)>`) need nothing: their ABI impls are unconditional.
- `&T` / `Option<&T>` arguments are `T: RefIntoWasmAbi`, by-value `T` is `T: IntoWasmAbi`, `&[T]` is
  `T: VectorRefIntoWasmAbi`, and returns are `T: FromWasmAbi`, all inline. `RefIntoWasmAbi` is per type, like
  upstream's `RefFromWasmAbi`: the macro implements it for each imported type, and wasm-bindgen for `JsValue`,
  strings and primitives. There is no `&T` blanket (`5314b95a1`).
- `Option<T>` and `Vec<T>` returns are bounded whole, in a first-order `where Option<T>: FromWasmAbi`.
  `T: OptionFromWasmAbi` would be inline but is the helper of upstream's `Option<T>` blanket, and it excludes
  `f64` (`KVNamespaceGetWithMetadataResult<_, f64>::metadata()`).

Counts over every fn signature in the crate's rustdoc (`for<'__wbg>` was the macro's, `WasmDescribe` every
synthesized bound's), measured at `527c0e0`. The total grows because each of the 299 imported types now lists
its own `RefIntoWasmAbi` and the restored upstream `IntoWasmAbi` / `OptionIntoWasmAbi for &Foo` impls (4 fns
each), less the setters the null model removed:

| in rustdoc signatures | before | after |
|---|---:|---:|
| `WasmDescribe` | 1,879 | 0 |
| `for<'` | 1,104 | 0 |
| `+ IntoWasmAbi` | 925 | 0 |
| `IntoWasmAbi` (not `RefIntoWasmAbi`) | 1,748 | 0 |
| `RefIntoWasmAbi` | 53 | 15 |
| `FromWasmAbi` | 131 | 5 |
| signatures with an ABI `where` | 808 | 5 |
| fn signatures (total) | 31,802 | 32,575 |

```rust
// before
pub fn new_with_str_and_init(input: impl JsStringLike + IntoWasmAbi + WasmDescribe, init: &RequestInit<Cf>)
    -> Result<Request<JsValue, Cf>, JsException>
    where for<'__wbg> &'__wbg RequestInit<Cf>: IntoWasmAbi + WasmDescribe, Request<JsValue, Cf>: FromWasmAbi +
        WasmDescribe,
// after
pub fn new_with_str_and_init(input: impl JsStringLike, init: &RequestInit<Cf>)
    -> Result<Request<JsValue, Cf>, JsException>
```
```rust
// before
pub fn get_with_metadata_typed<Metadata>(&self, key: impl JsStringLike + IntoWasmAbi + WasmDescribe)
    -> Promise<KVNamespaceGetWithMetadataResult<String, Metadata>, Error>
    where Promise<KVNamespaceGetWithMetadataResult<String, Metadata>, Error>: FromWasmAbi + WasmDescribe,
// after
pub fn get_with_metadata_typed<Metadata>(&self, key: impl JsStringLike)
    -> Promise<KVNamespaceGetWithMetadataResult<String, Metadata>, Error>
```
```rust
// before
pub fn put_with_buffer_source(&self, key: impl JsStringLike + IntoWasmAbi + WasmDescribe, value: impl
    BufferSourceLike + IntoWasmAbi + WasmDescribe)
    -> Promise<Option<R2Object>, Error>
// after
pub fn put_with_buffer_source(&self, key: impl JsStringLike, value: impl BufferSourceLike)
    -> Promise<Option<R2Object>, Error>
```
```rust
// before
pub fn put<T>(&self, key: impl JsStringLike + IntoWasmAbi + WasmDescribe, value: &T)
    -> Promise<(), Error>
    where for<'__wbg> &'__wbg T: IntoWasmAbi + WasmDescribe,
// after
pub fn put<T: RefIntoWasmAbi>(&self, key: impl JsStringLike, value: &T) -> Promise<(), Error>
```
```rust
// before
pub fn get_random_values<T>(&self, buffer: &T)
    -> Result<T, Error>
    where for<'__wbg> &'__wbg T: IntoWasmAbi + WasmDescribe, T: FromWasmAbi + WasmDescribe,
// after
pub fn get_random_values<T: RefIntoWasmAbi + FromWasmAbi>(&self, buffer: &T) -> Result<T, Error>
```
```rust
// before
pub fn cf(self, val: &Cf) -> Self where for<'__wbg> &'__wbg Cf: IntoWasmAbi + WasmDescribe,
// after
pub fn cf(self, val: &Cf) -> Self where Cf: RefIntoWasmAbi,
```

What is left in rustdoc:
- 5 `where` clauses, all first-order: 4 builder methods (`pub fn type_(self, val: &Type) -> Self where Type: RefIntoWasmAbi`),
  where the bound on the builder's own parameter stays method-level so the other builder methods remain callable
  for a `Type` with no ABI; and `SyncKvStorage::get<T>` (`where Option<T>: FromWasmAbi`). The getters' and helpers'
  bounds are hoisted to the `impl` header (`impl<R> ReadableStreamReadResult<R> where Option<R>: FromWasmAbi`).
- A hoisted bound sits on its own `impl` header, so methods with different bounds are listed under different
  headers (`impl<O: RefIntoWasmAbi> TransformStreamDefaultController<O>` beside the unbounded one).
- `respond_with_new_view(view: impl BufferSourceLike + AsRef<JsValue>)`: the `{@slice handle}` restriction, which
  excludes `&[u8]` because the view is transferred back. It is not an ABI bound.

**Null positions.** The null model (§7) as counted at `09a5580` and now:

| | before | after |
|---|---:|---:|
| `Option<impl JsStringLike>` arguments | 15 | 4 |
| `Option<&…>` arguments | 18 | 3 |
| public `_with_null` forms | 5 | 22 |
| `JsNullable<…>` (type arguments only) | 0 | 10 |
| `-> Option<JsValue>` reads (`null` folds to `None`) | 8 | 12 |
| setters | 688 | 598 |
| dictionary builders | 108 | 91 |
| port `None::<…>` | 0 | 0 |
| port `.set_x(Some(…))` | 3 | 0 |
| port `.is_null()` | 5 | 3 |

The 37 output-only types lose 100 setters, 18 constructors and 17 builders; the written nullable
fields add 10 hidden `null` setters (688 − 100 + 10 = 598). The remaining `Option<&…>` arguments are
`| undefined` positions (`File(bits: X[] | undefined, …)`, `Response.cf: any | undefined`). The port's `.is_null()` calls left are on
`any` values (a SQL cell, two D1 test assertions).

Every item that is **still worse in B than in A**:
- **`&[String]` → `&[impl JsStringLike]`:** `&[]` needs `&[] as &[&str]`. Literal lists and `&Vec<String>` both work.
- **`Option<&str>` → `Option<impl JsStringLike>` (4, from 15 before the null model):** two raw `| undefined` setters keep it by design (`EmailAttachment::set_content_id`, `UnderlyingSource::set_type`), and the two `new_inline*` factories by a ts-gen bug (§6). `None` there needs `None::<&str>`. No nullable position takes `Option`: each has a `_with_null` form.
- **Generic user code over a binding** restates the binding's declared bound: `T: RefIntoWasmAbi` for a `&T` argument, `T: FromWasmAbi` for a return, `Option<T>: FromWasmAbi` / `Vec<T>: FromWasmAbi` for those returns. All are first-order (phase 1 had 66 `for<'a>` clauses, phase 2 before #5385 had 12). A needed `T: JsGeneric`, which still covers a `&T` argument (`JsGeneric: RefIntoWasmAbi`).
- **Surface size.** The added items are:
  - `_typed` twins (51), for constructors and nested return-only parameters;
  - public `_with_null` forms (22), one per written nullable position;
  - KV typed reads. With the `_promise` twins gone, `KVNamespace` has 73 get/put/list/delete fns, against 70 in A and 78 in B1 (138 while async members had `_promise` twins). The longest is `get_with_metadata_text_with_options_typed_js_string`.

  Twins are down to 15, all intended (`text()` and KV values), and every `JsString` mention left is in their signatures.
- **Surface outside `bindings.rs` (unstable js-sys):** `Reflect::get` → `Result<Option<T>>`, `Array::get` → `Option<T>`, `JSON::stringify` → `Option<String>`, and iterators that rethrow instead of yielding `Result`. Code written against stable js-sys does not compile under the cfg (`worker`, `worker-sys`, serde-wasm-bindgen, wasm-streams).

## 4. Costs

**Size.** Each state is built from its own commit with its own wasm-bindgen pin, on the same
machine. The benchmark worker is a release build run through `wasm-opt -Oz` with
`benchmark/build.sh`'s flags. The test worker is a debug build and covers the whole suite's
surface. gz is `gzip -9`. B's sizes were measured at `527c0e0` and its compile times at `b901d43`; the commits between change bounds, null lowering and the `DataKind` arm. The null model shrank the benchmark by 213 B and one import (the `ResponseInit` headers `null` guard). A was re-measured for this report and
reproduces phase 1's numbers exactly. B1 is phase 1's measurement.

| | A | B1 | B | B − A |
|---|---:|---:|---:|---:|
| benchmark `index_bg.wasm` | 120,645 (51,600 gz) | 121,251 (51,735 gz) | 121,278 (51,653 gz) | +633 (+0.5%), +53 gz |
| benchmark `index.js` | 33,871 (6,371 gz) | 35,165 (6,449 gz) | 35,516 (6,334 gz) | +1,645 (+4.9%), -37 gz |
| benchmark wasm imports | 77 | 77 | 79 | +2 |
| test worker glue | 175,754 | 198,024 | 185,123 | +9,369 (+5.3%) |
| test worker wasm imports | 472 | 488 | 492 | +20 |
| test worker generic shims (`// generic import`) | 0 | 177 | 0 | +0 |
| test worker `finally` blocks (owned-string frees) | 10 | 14 | 6 | -4 |

**Compile time** of the `ts-gen-worker` crate from clean (`cargo clean -p ts-gen-worker`, deps
warm, wasm32). The table gives the second of two runs, with A and B measured back to back. B1 is
phase 1's measurement.

| | A | B1 | B | B − A |
|---|---:|---:|---:|---:|
| debug | 14.2 s | 17.6 s | 14.9 s | +0.7 s (+5%) |
| release | 11.1 s | 13.9 s | 11.8 s | +0.7 s (+6%) |
| test crate alone on top (debug) | 4.1 s | 4.3 s | 4.1 s | +0.0 s |

What moved since phase 1 (B1 → B):
- **Generic shims 177 → 0, glue +22,270 B → +9,369 B over A.** J's wire-format grouping makes `&str`, `String`, `&String` and `JsString` one import, and handles one import whether passed by value or by reference. The `// generic import` comments are gone without `--keep-debug`. Owned strings cross borrowed, so the `finally` frees went from 14 to 6 (A has 10).
- **Twins 383 → 15.** `{@string copy}` on small strings means less surface. The output does not depend on it, because unused twins never reach the binary.
- **Compile time +23–25% → +5–6% over A.** One `Promise<T, crate::Error>` return per async member removed every `async fn` wrapper and `_promise` twin, and one `impl BufferSourceLike` argument the buffer arm pairs. `bindings.rs` is now 20,149 lines, against A's 22,148 and B1's 23,740.

What B still costs over A:
- **Benchmark: +2 imports, +0.5% wasm (gzip +0.1%).** The import diff, with A's and B's import bodies normalised, is three swaps, three additions and one removal:
  - `Error::name` and `Error::message` return a `String` copy instead of a handle (two swaps);
  - `ResponseInit::headers` is read as `Option` (a swap);
  - string bodies cross as UTF-8 (`new Response(getStringFromWasm0(..), init)`) beside the handle form the stream and `null` bodies use;
  - awaiting a `Promise<()>` resolves through a closure typed `fn(())`. js-sys's `IntoFuture` for `Promise<T>` makes one closure cast per awaited wire format;
  - `Promise.reject`, which `catch` on a `Promise<T>` import uses to return a synchronous throw;
  - removed: A's `=== null` check on the `ResponseInit` headers read, since an `Option<JsValue>` read folds `null`.
- **Test worker: +20 imports, +5.3% glue.** The added imports are distinct wire formats rather than per-type shims:
  - string returns copied into Rust (`toISOString`, `toString`, tuple and record values) where A handed back a `JsString`;
  - string arguments as UTF-8 where A passed a handle (`body`, `prefix`, `type`);
  - byte arguments as wasm-memory views (`put`, `uploadPart`) beside the handle form;
  - awaited wire formats (`()`, `Option<f64>`, `Option<ArrayBuffer>`, …): 6 more closure casts;
  - the port's wider API use: per-type KV reads, `get_with_metadata_typed::<f64>`, generated `[Symbol.iterator]`, `Object::from_iter`/`collect` (`Reflect.set` by string key), three `Function::from_fn` closures.
- **Compile time** in the bindings crate is +5% debug and +6% release, though `bindings.rs` is smaller than A's. That is the mono macro's larger expansion per generic binding. The test crate on top is flat. An earlier back-to-back pair under more load came out −9% / +1%, so the delta is near this machine's noise.

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
| `slice` modes on a generic mono argument, per instantiation | wasm-bindgen macro | 2 | yes |
| `Promise<T, E = JsValue>`, awaiting to `Result<T, E>` with `E` erased for the ABI | js-sys | 2 | no |
| `String` returns from built-ins (`Error`, `Date`, `JSON`, `Intl`); `Date::new_with_value`; `ArrayBuffer` bytes | js-sys | 2 | no |
| iterators yield `T`; `Entries<K, V>`; Map/Set/Object as Rust collections | js-sys | 2 | no |
| phase 1's five hand-applied `mono-patch` stopgaps replaced by ts-gen fixes | ts-gen | 2 | yes |
| `{@string copy\|ref\|both}`; `@retype`; side namespaces; a side `readonly` drops the setter (31 properties workerd defines without one) | ts-gen + `annotations.d.ts` | 2 | no |
| `_typed` twins (constructors, nested return-only params); concrete construct returns | ts-gen | 2 | yes |
| null model: a written `T \| null` (argument, setter, builder, field) is the bare value plus a `_with_null` form; output-only types are read-only; reads fold `null` into `Option`; nullable type arguments are `JsNullable<T>` | ts-gen + js-sys (`JsNullable<T>` for any ABI `T`) | 2 | partly |
| callbacks with Rust leaves, `Promise<()>` | ts-gen | 2 | yes |
| async methods return `Promise<T, crate::Error>`, with no `async fn` / `_promise` pair | ts-gen | 2 | partly |
| bounds are the declaration's: inline `T: RefIntoWasmAbi` / `T: FromWasmAbi`, first-order `Option<T>: FromWasmAbi`, nothing on `impl …Like` or concrete types | wasm-bindgen (#5385, per-type `RefIntoWasmAbi`) + ts-gen | 2 | yes |
| `Iterable<T>` → slices; alias leaves; optional `any` → `Option<JsValue>`; generated `IntoIterator` | ts-gen | 2 | no |
| per-literal overload bindings (KV typed reads); literal-only dictionaries set by the binding (`raw_with_column_names`, `get_reader_byob`); shorter overload and inline-type names | ts-gen | 2 | no |
| `ArrayBuffer \| ArrayBufferView` → `impl BufferSourceLike`, copied unless `{@slice view}`, handles only for `{@slice handle}`; `this` returns the receiver; optional `Partial` getters | ts-gen + `annotations.d.ts` | 2 | partly |
| rest tuples lower to arrays; inline types reached only by an omitted argument are not emitted | ts-gen | 2 | no |
| a nullable handle-or-primitive union return is `Option` of its enum; `MessageEvent.data` retyped | ts-gen + `annotations.d.ts` | 2 | no |
| bare generic-class arguments take any instantiation | ts-gen | 2 | yes |
| `M[K]` over a concrete `M` binds once per key (typed event listeners on subclasses, with the options and `bool` arms) | ts-gen | 2 | yes |

C would need instead (none built):
- wasm-bindgen#5167 with the root `&T` slot leak fixed;
- `JsStr<'a>` and a sealed `JsStringLike::as_js_str`, plus a list either type;
- tagged enums with `From` impls for the 12 primitive unions;
- hand-written `IntoJsGeneric`/`FromJsGeneric` for every user type used as a type argument.

Every "no" row carries over to C as is. The js-sys collection behaviour would carry over too,
along with the same ecosystem break.

## 6. Remaining gaps

Not built, by owner. Every review finding not listed here is done.

**Ecosystem (workers-rs; needs design, outside the prototype).**
- `worker` and `worker-sys` are not ported to `js_sys_unstable_apis`, so the repo's 145-test `chomp test` stays excluded. B's js-sys changes behaviour, not just types:
  - iterators rethrow instead of yielding `Result`;
  - `Reflect::get` / `Array::get` return `Option`;
  - `JSON::stringify` returns `Option<String>`;
  - `Map::get` borrows its key.

  Shipping B means porting `worker` (and depending on patched serde-wasm-bindgen and wasm-streams), or stabilising these js-sys APIs first.
- Every patch is prototype quality. `mono-prototype-patches.md` groups them by upstream PR.

**ts-gen.**
- Output-only misses parents and intersection members. `D1Response`, the parent of output-only `D1Result`,
  keeps `new(meta)`, `set_success` and `set_meta`, reachable on any `D1Result` through `Deref`. `D1Meta` keeps a
  dead write side, since `meta: D1Meta & Record<…>` lowers to `JsValue`; typing it as `D1Meta` needs an
  intersection rule.
- A union of slice elements with no Rust form is erased: `AnalyticsEngineDataPoint.{indexes,blobs}` take
  `&[JsValue]`, so the port writes `.blobs(&[req.method().into(), …])`, and `File::new`'s `bits` is the same.
  Expanding an element union like an argument union (`&[impl JsStringLike]`) would remove the last `.into()`s.
- Shape-shared union enums are named after their first site: R2's `ssec_key()` returns
  `MessageEventInitDataKind` (§7).
- `[A, ...B[]]` lowers to an erased array: D1 `raw_with_column_names()` returns `Vec<JsValue>`, the column names followed by the rows. A and B1 typed it as a fixed 2-tuple, which was wrong. A typed head/rest form needs a js-sys type.
- `IncomingRequestCfProperties` has numbered builders, from crossing a dictionary-union field with the type's other unions. Only mocks build an incoming `cf`.
- `EmailAttachment::new_inline(content_id: Option<…>)`: the `inline` variant requires `contentId`, but the
  branch factory takes the merged field's `string | undefined`, so it takes `Option`.
- Eight Promise-typed getters (`writer.ready()`, `socket.opened()`, the readers' and writers' `closed()`) return
  `Promise<T>`, which rejects with `JsValue`, so a tail `.await` of one is `Ok(x.await?)`. Only methods were given the crate's error type.

**ts-gen + wasm-bindgen.**
- `FormData::get_all()` stays `Vec<JsValue>`: a `Vec<T>` return needs a handle `T`, and a union enum is not one.

**wasm-bindgen / js-sys.**
- Array callback methods other than `for_each` (`map`, `filter`, …) stay erased. Rust iteration covers them.

**By design (no action).**
- `&[] as &[&str]`. `None::<&str>` on the two raw `| undefined` setters (`EmailAttachment::set_content_id`,
  `UnderlyingSource::set_type`). The other two `Option<impl JsStringLike>` arguments are the `new_inline*` factories (ts-gen, above).
- `any` values stay `JsValue` (SQL values, attachments, `Error::cause`).
- `void | Promise<void>` callback returns are `Option<Promise<()>>`. That is correct, but a hand-built handler closure returns `None`/`Some(p)`; the macros hide it.
- Mono's compile-time cost (§4).

## 7. Decisions taken during the prototype (reversible, for review)

Policies chosen while building B to finish a port idiom, flagged for review. Each can be reverted in
ts-gen or `annotations.d.ts`.

- **Return-only type parameters.** A method whose return is exactly `T` or `T | null` keeps the
  generic, so the turbofish is the cast (`storage.get::<f64>(k)`, like `parse::<T>()`). When the parameter is
  nested in the return (`getWithMetadata<Metadata>`, `list<Metadata>`), the default form erases it and a
  `_typed::<T>` twin binds it, the same suffix constructors use. Keeping the generic everywhere was rejected:
  Rust fns have no default type parameters, so `kv.list().await?` would need a turbofish with metadata unused.
- **Async methods return `Promise<T, crate::Error>`.** Each async method is one binding returning a
  typed promise, with `T` the awaited form (`Promise<Option<String>, crate::Error>`, `Promise<(), crate::Error>`).
  There is no `async fn` / `_promise` pair: the 293 `_promise` twins are gone and public fns went from 3,628 to
  3,335. js-sys's `Promise<T, E = JsValue>` is `IntoFuture<Output = Result<T, E>>` with `E` erased for the ABI.
  So `x.await` gives `Result<T, crate::Error>` as the `async fn`s did, tails and `x.await?` are unchanged, and the
  same value passes to JS (`p.upcast()` for a `&Promise` parameter). The imports keep `catch`: on a promise return,
  wasm-bindgen hands a synchronous throw back as a rejected promise, so it surfaces at `.await`.

  A first cut returned `Promise<T>`, which rejects with `JsValue`. That made 37 tail `x.await`s `Ok(x.await?)` until
  js-sys gained the error parameter. The other alternative, a ts-gen-worker wrapper with `Output = Result<T, Error>`
  and an `into_promise()`, would add a type to every async return.
- **Typed event listeners per key.** `addEventListener<K extends keyof M>(type: K, handler: (e: M[K]) => …)`
  on a class whose `M` is a declared interface gets one binding per key, with the type string set by the binding:
  `WebSocket::add_event_listener_message(&Function<fn(MessageEvent)>)`. Subclasses declare them for their first
  parent's methods, and the generic string form stays. Overloads told apart by one literal are now named by it.
  The alternative was an `on_message(impl FnMut(MessageEvent))` form that owns the closure. That is shorter at the
  call site but hides the `Function` that `removeEventListener` needs.
- **The null model.** Writes keep `null` exact as an ordinary union arm, reads fold
  both empties into `Option`, and type arguments (no direction) keep `JsNullable<T>`. No `None` sends
  `undefined` where the declaration says `null`, so ts-gen#51's no-coercion rule holds. As built:

  | position | B |
  |---|---|
  | argument, setter, builder value, required field `T \| null` | the bare value plus a `_with_null` form: `clear_timeout(id)` / `clear_timeout_with_null()`, `set_prefix(p)` / `set_prefix_with_null()`, `.prefix(p)` / `.prefix_with_null()`, `Response::new_with_null_and_init(&init)` |
  | `any \| null` argument | `&JsValue` (`JsValue::NULL` is a value) |
  | optional `x?: T` | unchanged: `Option<T>` or an omitted overload |
  | output-only types | getters only (37 types: KV list result and key, `KVNamespaceGetWithMetadataResult`, Queue metrics, `R2Objects`, `SocketInfo`, …) |
  | return, getter, awaited value, callback argument | `Option<T>`, `JsValue` included: `null` and `undefined` are both `None` (`deserialize_attachment()` → `Result<Option<JsValue>, Error>`) |
  | type argument `T \| null` | `JsNullable<T>` (KV bulk get `Map<String, JsNullable<String>>`) |
  | type argument `T \| undefined` | `Option<T>` |
  | returned `Vec<T \| null>` | `Vec<Option<T>>` |

  `Option<JsValue>` reads always folded `null` (the glue is `isLikeNone(ret) ? 0 : …`), so the port's KV metadata
  filter and `ResponseInit` headers guard were dead code; both are gone.
  Reversible within the model: `any | null` as `&JsValue` rather than a `_with_null` form, and `Vec<T | null>`
  returns folded to `Option` rather than kept as `JsNullable`.
- **An optional parameter's trailing `null` arm is dropped.** Omitting the argument covers it. That is exact
  for WebIDL `T? = null`; a non-WebIDL callee that tells missing from `null` loses the bare-`null` call.
- **Buffer sources copy by default.** A `&[u8]` buffer-source argument is copied into a fresh `Uint8Array`
  unless the declaration says `{@slice view}`. The view is set only where the callee copies or reads before
  returning (`Response` body, KV/R2 `put`, R2 `uploadPart`, WebCrypto: 29 arguments), so new declarations fail
  safe. The alternative, a view by default with copies opted in, saves a copy at unannotated call sites but can
  hand a transferring or storing callee a view of wasm memory.
- **`MessageEvent.data` is `string | ArrayBuffer | Blob | any`.** Upstream types it `any`. A handle-or-primitive
  union lowers to a generated enum (`typeof` / `instanceof` dispatch), and a nullable one to `Option` of it
  (`FormData::get` → `Option<GetReturnKind>`). The `Blob` arm is needed: from the 2026-03-17 compat date
  (`websocket_standard_binary_type`) binary messages are Blobs unless `binaryType` is set. The trailing `any` is a
  catch-all `JsValue` arm, since a `MessagePort` message is any structured clone. Without it an unmatched value
  throws `invalid dynamic union value` inside the getter, where Rust cannot catch it
  (`test/tests/worker.spec.ts` "MessageEvent data outside …"). The enum takes the bare name `DataKind`, which the
  Durable Object `websocket_message` handler shares, so that handler sees `Blob` and `JsValue` arms that
  hibernation never delivers. The two-arm shape is now `MessageEventInitDataKind` (event construction, R2 `ssecKey`).
- **Optional properties typed only `never` or `undefined` have no accessors.** This removes
  `D1Result::error()`, which `worker` exposes as `Option<String>`.

