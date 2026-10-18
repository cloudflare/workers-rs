# Generic mono end state (B) vs today (A) vs a non-mono end state (C)

A = non-mono today (`a4cbe97` bindings, `104579e` port). B = mono + `js_sys_unstable_apis` + js-sys
flesh-out + the known fixes, built on `gbedford/tgw-mono-endstate` (`14be4c3` build, `2aa55cc`
idiomatic port; ts-gen `87882e1`, wasm-bindgen `4fe6881c0`, all local). C = non-mono +
wasm-bindgen#5167 + a `JsStr<'a>` either type behind `impl JsStringLike` + tagged enums for
primitive unions: **design, not built**. Its column is written by hand from the #5167 compile
matrix (`~/Projects/wasm-bindgen-5` `2497681c1`, stable js-sys). S15 = session-15 mono (`a806700`).

## Verdict

1. B builds and passes the suite (194/194 plain and `-http`). Its port is 58 lines shorter than A's, has no `JsString` left, and has a third fewer `unchecked_*` casts and half the `String::from(js)` calls.
2. B mixes two independent axes. The container reads (`Map<String, _>`, `ArrayTuple<(String, String)>`, `Object<String>`, `Promise<String>`) come from `js_sys_unstable_apis` plus the js-sys flesh-out, and a non-mono build can enable that too: non-mono imports returning these types compile (checked in a scratch crate). Mono's own gains are leaf type arguments on generated generics (`get::<f64>`, `put(&f64)`, `first::<f64>`) and primitive unions as one function.
3. C as specified (stable js-sys) matches B at string arguments and gets most container reads through #5167. It loses where keys must be exact (`get(&"k".to_string())`), on `Option<T>`-shaped returns (`get::<JsValue>().as_f64()` stays), and where js-sys's own signatures say `JsString`.
4. B costs +23–24% `ts-gen-worker` compile time (a third of that is the `_js_string` twins), +3.8% benchmark glue, +0.5% wasm and +16 test-worker imports. It needs the five session-15 stopgaps plus one extension and opens one new hole (`Map<_, Option<_>>::get`). `js_sys_unstable_apis` also breaks `worker`/`worker-sys`, serde-wasm-bindgen and wasm-streams for the whole build.
5. B is the best surface measured, but most of its lead over C is the unstable-js-sys axis, which is also its largest cost. The mono-only lead is the leaf-generic cases in (2).

**Built for B**: every item in the checklist marked *built* (§5); the port rewrite; the full suite.
**Assumed for B** (not built): `Option<impl JsStringLike>` / `Option<f64>` for `T | null` arguments
(24 `&JsNullable<JsString>` and 5 `&JsNullable<Number>` remain); constructors returning the TS
return type's concrete arguments (`ReadableStream<Uint8Array>` for a byte source); an `Option<Option<T>>`
ABI; typed `Promise::new` resolvers; `worker`/`worker-sys` under `js_sys_unstable_apis` (excluded,
so the 145-test `chomp test` was not run).

## 1. End-state examples

Each A block cites its source at `104579e` (paths under `ts-gen-worker/`; `bindings.rs` lines are
`a4cbe97`). B blocks are the committed port (`2aa55cc`) unless marked *signature only*. C blocks
are hand-written.

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
// B
headers.set("Access-Control-Max-Age", max_age.to_string())?;            // impl JsStringLike
pub struct HeadersIter(IntoIter<ArrayTuple<(String, String)>>);
Some(self.0.next()?.ok()?.into_tuple())
let keys = headers.keys().into_iter().collect::<Result<Vec<_>, _>>()?.join(",");
```
```rust
// C
headers.set("Access-Control-Max-Age", max_age.to_string())?;            // impl JsStringLike -> JsStr
pub struct HeadersIter(IntoIter<ArrayTuple<(String, String)>>);         // String: JsGeneric (#5167)
Some(self.0.next()?.ok()?.into_tuple())
let keys = headers.keys().into_iter().collect::<Result<Vec<String>, _>>()?.join(",");
```
B gains `String` tuple and iterator items. All three keep the `Result` per item (the JS iterator
protocol) and so the `collect::<Result<…>>` turbofish. C lands on the same call sites.

### Request / Response, init builder, `cf`

```rust
// A: src/ext/interop.rs:84-97
let mut init = RequestInit::builder().method(req.method().as_str()).headers(&Headers::try_from(req.headers())?);
if let Some(cf) = req.extensions_mut().remove::<IncomingRequestCfProperties>() {
    init = init.cf(&cf);                                                 // cf(self, val: &Cf)
}
if let Some(body) = body_stream(req.into_body()) {
    init = init.body(body.unchecked_ref());                              // &JsNullable<ReadableStream<Uint8Array>>
}
Ok(Request::new_with_str_and_init(&uri, &init.build())?)
```
```rust
// B: identical. `cf(self, val: &Cf)` is A's again (S15 took `val: Cf` by value), and
// `new_with_str_and_init<Cf>` binds only `Cf` (S15 also bound an uninferable `CfHostMetadata`).
init = init.cf(&cf);
init = init.body(body.unchecked_ref());
Ok(Request::new_with_str_and_init(&uri, &init.build())?)
```
```rust
// C: identical to A.
```
B gains nothing over A here: patterns 6 and 7 undo two S15 regressions. All three keep
`unchecked_ref` into the nullable union argument.

### Body from string / bytes / stream

```rust
// A: test/src/lib.rs:153-164, src/ext/response.rs:107-118
Response::new_with_array_buffer_and_init(&JsNullable::wrap(out.buffer()), &init)?
Response::new_with_str(&JsNullable::wrap(format!("{a}|{b}").into()))?  // body: &JsNullable<JsString>
Response::new_with_str_and_init(&JsNullable::new(), &init)?             // empty body
Response::new_with_readable_stream_and_init(&JsNullable::wrap(stream.unchecked_into()), &init)?
```
```rust
// B: identical; `JsNullable::wrap` 42 -> 42 in the port.
Response::new_with_str(&JsNullable::wrap(format!("{a}|{b}").into()))?
```
```rust
// C (design): `string | null` as Option<impl JsStringLike>, a nullable handle as Option<&T>
Response::new_with_str(Some(format!("{a}|{b}")))?
Response::new_with_str_and_init(None::<&str>, &init)?
Response::new_with_array_buffer_and_init(Some(&out.buffer()), &init)?
```
Mono lowers `string` and `string | undefined` arguments but not the `T | null` arguments a union
expansion produces. That leaves 24 `&JsNullable<JsString>` and 5 `&JsNullable<Number>`. The C
lowering is a ts-gen change available to B too (assumed, not built), and it brings `None::<&str>` with it
in both.

### KV (get, metadata, bulk get → `Map`, put options, list)

```rust
// A: test/src/kv.rs:164-176, 181, src/ext/kv.rs:106, test/src/kv.rs:116, 154-158
async fn get_bulk_text(store: &KVNamespace, keys: &[&str]) -> Result<js_sys::Map> {
    let keys: Vec<String> = keys.iter().map(|k| (*k).to_owned()).collect(); // key: &[String]
    Ok(store.get_with_slice(&keys).await?.unchecked_into())                  // Map<JsString, JsNullable<JsString>>
}
let values = get_bulk_text(&store, &["bulk_a", "bulk_b", "bulk_missing"]).await?;
kv_assert_eq!(values.get(&"bulk_a".into()).as_string(), Some("value_a".to_string()))?;
let values = get_bulk_text(&store, &[]).await?;
KVNamespaceGetOptions::builder(&JsString::from(value_type))
KVNamespaceListOptions::builder().prefix(&JsNullable::wrap("put_c".into()))
let result = store.get_with_metadata("put_d").await?;                        // <JsString, JsValue>
kv_assert_eq!(result.value().unwrap(), "test")?;
```
```rust
// B: test/src/kv.rs:169-195, src/ext/kv.rs:110, 281-288
let values: HashMap<_, _> = store
    .get_with_slice(&["bulk_a", "bulk_b", "bulk_missing"])                   // &[impl JsStringLike]
    .await?                                                                  // Map<String, Option<String>>
    .into_iter()
    .collect();
kv_assert_eq!(values["bulk_a"], Some("value_a".to_string()))?;
// values.get("bulk_a") does not compile: Option<Option<String>> has no FromWasmAbi
let values = store.get_with_slice(&[] as &[&str]).await?;                    // element type needed
KVNamespaceGetOptions::builder(&value_type.to_string())                      // `&str` would infer T = str
KVNamespaceListOptions::builder().prefix(&JsNullable::wrap("put_c".into()))  // unchanged
fn collect<V: FromWasmAbi, T>(map: Map<String, V>, mut decode: impl FnMut(V) -> Result<T>) -> Result<HashMap<String, T>> {
    map.into_iter().map(|(key, value)| Ok((key, decode(value)?))).collect()  // A: for_each + failure slot
}
```
```rust
// C
let values = store.get_with_slice(&["bulk_a", "bulk_b", "bulk_missing"]).await?; // Map<String, Option<String>>
kv_assert_eq!(values.get(&"bulk_a".to_string()), Some("value_a".to_string()))?;  // stable get -> V; key exactly &String
let values = store.get_with_slice(&[] as &[&str]).await?;
KVNamespaceGetOptions::builder(&value_type.to_string())
KVNamespaceListOptions::builder().prefix(Some("put_c"))
```
B drops the helper, the cast and `.as_string()`. B gets wrong:
- `Map<_, Option<_>>::get` cannot compile, so you iterate.
- `&[]` needs its element type.
- A `&str` key into a `<String>` dictionary allocates.

C keeps `get`, because its `V` is bare (`Option<String>: JsGeneric`), but the key has to be `&String`.

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
R2ListOptions::builder().include(&["customMetadata".to_string()])            // &[String]
let prefixes: Vec<String> = objects.delimited_prefixes();
```
```rust
// B: test/src/r2.rs:74-88, 167
object
    .custom_metadata()                                                       // Option<Object<String>>
    .and_then(|record| Object::entries(&record).ok())                        // Array<ArrayTuple<(String, String)>>
    .map(|entries| entries.iter().map(|entry| entry.into_tuple()).collect())
    .unwrap_or_default()
fn record(map: &HashMap<String, String>) -> Object<String> {
    let obj = Object::new_typed();
    for (k, v) in map { Reflect::set(&obj, k, v).unwrap(); }
    obj
}
R2ListOptions::builder().include(&["customMetadata"])
```
```rust
// C
object.custom_metadata()                                                     // Option<Object<String>>
    .and_then(|record| Object::entries_typed(&record).ok())                  // Array<ArrayTuple<(JsString, String)>>
    .map(|entries| entries.iter().map(|e| { let (k, v) = e.into_tuple(); (String::from(k), v) }).collect())
    .unwrap_or_default()
fn record(map: &HashMap<String, String>) -> Object<String> {
    let obj = Object::new_typed();
    for (k, v) in map { Reflect::set(&obj, &k.into(), &v.into()).unwrap(); } // stable Reflect: &JsValue
    obj
}
R2ListOptions::builder().include(&["customMetadata"])
```
B's `Record<string, string>` reads and writes without a cast. C still leaks `JsString` because the
key type is fixed by js-sys's stable signature (`(JsString, T)`), not by ts-gen. `delimited_prefixes`
is `Vec<String>` in all three.

### D1 (bind mixed values, `first` by column, `all` / `raw`)

```rust
// A: test/src/d1.rs:16-20, 38-40, 57-61, bindings.rs:19788
let bindings = args.iter().map(|arg| arg.serialize(&serializer).map_err(|e| e.to_string()))
    .collect::<Result<Vec<JsValue>, String>>()?;
prepared.bind(&bindings)                                                     // values: &[JsValue]
let value: JsValue = match col_name {
    Some(col) => stmt.first_with_col_name::<JsValue>(col).await?,            // JsNullable<T>
    None => stmt.first::<JsValue>().await?,
}.into();
// raw_with_d1_prepared_statement_options -> ArrayTuple<(Array<JsString>, Array<Array>)>
```
```rust
// B: test/src/d1.rs:58 unchanged; the signature is now Option<T>
Some(col) => stmt.first_with_col_name::<JsValue>(col).await?,
let n: Option<f64> = stmt.first_with_col_name("n").await?;                   // signature only
// raw_with_d1_prepared_statement_options -> ArrayTuple<(Array<String>, Array<Array>)>
```
```rust
// C
Some(col) => stmt.first_with_col_name::<JsValue>(col).await?,                // JsNullable<T>
// stmt.first_with_col_name::<f64>("n") does not compile: T inside JsNullable<T> is a structural
// position, which needs T: ErasableGeneric<Repr = JsValue>; #5167's leaf path is bare T only.
```
`bind(&[JsValue])` (`unknown[]`) and serde rows are the same in all three. B allows a leaf `T` for
`first`. C does not.

### Durable Object storage (get / put / list → `Map`, bulk get, put entries)

```rust
// A: test/src/ext.rs:98-104, test/src/durable.rs:83-99, 168-178
let count = storage.get::<JsValue>("count").await?.as_f64().unwrap_or(0.0) + 1.0; // JsOption<T>
storage.put("count", &JsValue::from(count)).await?;                         // value: &T
for key in list.keys() { let key = String::from(key?); /* … */ }             // Map<JsString, JsValue>
let vals: js_sys::Map = storage.get_with_keys(&keys).await?.unchecked_into();
serde_wasm_bindgen::from_value::<Option<i32>>(vals.get(&"anything".into()))
storage.delete_with_keys(&["thing".to_string(), "other".to_string()]).await?;
let obj = js_sys::Object::new();
js_sys::Reflect::set(&obj, &JsValue::from_str("foo"), &value.into())?;
storage.put_with_entries::<JsValue>(&obj.unchecked_into()).await?;
```
```rust
// B: test/src/ext.rs:98-99, test/src/durable.rs:84-99, 167-175
let count = storage.get::<f64>("count").await?.unwrap_or(0.0) + 1.0;       // Option<T>
storage.put("count", &count).await?;
for key in list.keys() { let key = key?; /* … */ }                           // Map<String, JsValue>
let vals = storage.get_with_keys(&keys).await?;
let val = |key| vals.get(key).unwrap_or_default();                           // MapKey: &str for String keys
serde_wasm_bindgen::from_value::<Option<i32>>(val("anything"))
storage.delete_with_keys(&["thing", "other"]).await?;
let obj = js_sys::Object::new_typed();
js_sys::Reflect::set(&obj, "foo", &value)?;
storage.put_with_entries(&obj).await?;
```
```rust
// C
let count = storage.get::<JsValue>("count").await?.as_f64().unwrap_or(0.0) + 1.0; // JsOption<T>: no leaf path
storage.put("count", &count).await?;                                        // root &T via ref_to_js
for key in list.keys() { let key = key?; /* … */ }                           // Map<String, JsValue>
let vals = storage.get_with_keys(&keys).await?;
let val = |key: &str| vals.get(&key.to_string());                            // stable get -> V
storage.delete_with_keys(&["thing", "other"]).await?;
js_sys::Reflect::set(&obj, &"foo".into(), &value)?;
storage.put_with_entries(&obj).await?;
```
This is B's strongest area: `get::<f64>` and `put(&f64)` need mono, because the awaited
`T | undefined` lowers to `Option<T>`. C's `put(&count)` works but, as #5167 is written, it leaks
one externref slot per call. `get::<f64>` is out of reach for C.

### WebSocket hibernation (tags, `get_web_sockets(tag)`, `get_tags`)

The port accepts without tags (`test/src/counter.rs:40-41`); the tag APIs are signature only.
```rust
// A: bindings.rs:1207, 1217, 1255; test/src/counter.rs:41, 60-63
state.accept_web_socket_with_tags(&server, &["room".to_string()])?;          // tags: &[String]
let peers: Vec<WebSocket> = state.get_web_sockets_with_tag("room")?;
let tags: Vec<String> = state.get_tags(&ws)?;
server.serialize_attachment(&"hello".into())?;                              // attachment: &JsValue (any)
let attach: String = ws.deserialize_attachment()?.as_string().expect("…");
```
```rust
// B
state.accept_web_socket_with_tags(&server, &["room"])?;                     // &[impl JsStringLike]
let peers = state.get_web_sockets_with_tag("room")?;
let tags = state.get_tags(&ws)?;                                            // Vec<String> (+ get_tags_js_string)
server.serialize_attachment(&"hello".into())?;
```
```rust
// C
state.accept_web_socket_with_tags(&server, &["room"])?;                     // list either type
let peers = state.get_web_sockets_with_tag("room")?;
let tags = state.get_tags(&ws)?;
```
B and C both accept literal tag lists. Attachments stay `any` in all three.

### WebSocket send / close / message data

```rust
// A: test/src/counter.rs:67, test/src/container.rs:119-133, test/src/util.rs:61-62
ws.send_with_str(&format!("{count}"))?;                                      // message: &str
dst.close_with_code_and_reason(reason.code(), &reason.reason())?;
event.data().as_string()                                                     // data: any -> JsValue
```
```rust
// B: test/src/counter.rs:67, test/src/container.rs:119-133
ws.send_with_str(format!("{count}"))?;                                       // impl JsStringLike
dst.close_with_code_and_reason(reason.code(), reason.reason())?;
event.data().as_string()
```
```rust
// C: B's call sites; `String` and `&str` share one import through JsStr.
```
Borrowing becomes optional, and clippy then removes the `&` (12 sites). In B every Rust string type is
its own shim: `send_with_str(&s)` and `send_with_str(s)` are two imports (S15 feedback 9). C keeps
one import per JS function.

### Queues (send / sendBatch, message body)

```rust
// A: test/src/queue.rs:45-50, 69-75
let body = serde_wasm_bindgen::to_value(&QueueBody { id, id_string: id.to_string() })?;
my_queue.send(&body).await?;                                                 // message: &Body
let batch = js_sys::Array::new();
for message in &messages { batch.push(&MessageSendRequest::new(&serde_wasm_bindgen::to_value(message)?)); }
my_queue.send_batch(&batch).await?;                                          // messages: &JsValue
```
```rust
// B: identical (S15 had `send(body)` by value; pattern 6 restores `&Body`).
```
```rust
// C: identical.
```
None of the three improves on A here: `Iterable<MessageSendRequest<Body>>` erases to `&JsValue`.
A `Queue<String>` would take `send(&"x".to_string())` in B, and in C through `ref_to_js`.

### Streams (`TransformStream`, writer)

```rust
// A: test/src/cache.rs:125-141, test/src/socket.rs:28-30
let transform = TransformStream::new()?;                                     // TransformStream<JsValue, JsValue>
let writer = transform.writable().get_writer()?;
writer.write_with_chunk(&Uint8Array::from(data)).await?;                     // chunk: &W
Response::new_with_readable_stream_and_init(transform.readable().unchecked_ref(), &init)?
```
```rust
// B: identical. S15 needed `let transform: TransformStream<Uint8Array, Uint8Array> =` and moved
// the chunk (`write_with_chunk(chunk)`); patterns 7 and 6 remove both.
```
```rust
// C: identical.
```
A typed `TransformStream::<Uint8Array, Uint8Array>::new()` exists in none of the three (the
constructor binds no class parameter). The readable side still goes through `unchecked_ref` into
`&JsNullable<ReadableStream>`.

### Cache

```rust
// A: test/src/cache.rs:40-48
let key = req.url();
if let Some(resp) = cache.match_with_str_and_options(&key, &options).await? { /* … */ }  // request: &str
cache.put_with_str(&key, &resp.clone()?).await?;
```
```rust
// B: identical (`&String` is JsStringLike); `key` by value also compiles.
```
```rust
// C: identical.
```
No difference.

### Email (attachments, `Option<&str>`)

```rust
// A: test/src/send_email.rs:96-103, 133-137; bindings.rs:20152
let attachment = EmailAttachment::new_attachment_with_typed_array("hello.txt", "text/plain", &bytes);
EmailMessageBuilder::builder(SENDER, subject).to(RECIPIENT).attachments(&[attachment])
EmailMessage::new_with_str(from, to, &scenario.raw())?
EmailAttachment::builder(/* … */).content_id(None)                           // val: Option<&str>
```
```rust
// B: test/src/send_email.rs:135; content_id is signature only
EmailMessage::new_with_str(from, to, scenario.raw())?
EmailAttachment::builder(/* … */).content_id(None::<&str>)                   // Option<impl JsStringLike>
EmailAttachment::builder(/* … */).content_id(Some("logo"))
```
```rust
// C: Option<impl JsStringLike> gives the same None::<&str>; a concrete Option<JsStr<'_>> instead
// gives content_id(None) and content_id(Some("logo".into())).
```
For B, this is a regression from A on all 7 `Option<impl JsStringLike>` arguments. C has to choose
which call site pays: `None::<&str>`, or `.into()` on `Some`.

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
span.set_attribute("retries", 3)?;                                           // any int, f64, bool, &str, String, JsString
span.set_attribute("route", "/api")?;
```
```rust
// C: value: impl Into<BooleanOrNumberOrString<'_>>, a tagged enum
span.set_attribute("retries", 3.0)?;
```
B's 13 union sites are one function each, and every Rust type is its own shim. C gets the same
call sites from one concrete import, provided ts-gen emits the enum's `From` impls. Integer
literals then need `From<i32>` and friends.

### Env vars and secrets

```rust
// A: src/ext/env.rs:29-41, test/src/alarm.rs:50-51, test/src/form.rs:139-140
let value = js_sys::Reflect::get(env, &JsValue::from_str(name))?;
if value.is_undefined() { return Err(Error::Rust(format!("Binding `{name}` is undefined."))); }
let namespace: DurableObjectNamespace = js_sys::Reflect::get(&env, &"ALARM".into())?.unchecked_into();
let val = js_sys::Reflect::get(&env, &name.into())?;
val.as_string().unwrap_or_default()
let secret: String = env.secrets().get().await?;                             // SecretsStoreSecret::get
```
```rust
// B: src/ext/env.rs:30-31, test/src/alarm.rs:50-52, test/src/form.rs:139-140
js_sys::Reflect::get(env, name)?.ok_or_else(|| Error::Rust(format!("Binding `{name}` is undefined.")))
let namespace: DurableObjectNamespace =
    js_sys::Reflect::get(&env, "ALARM")?.unwrap_or_default().unchecked_into();
let val = js_sys::Reflect::get(&env, name)?;
val.and_then(|v| v.as_string()).unwrap_or_default()
```
```rust
// C: A's form (stable js-sys Reflect).
```
This churn comes from unstable js-sys, not from mono. `&str` property keys are better, but
`Result<Option<T>>` adds a step wherever the old code ignored `undefined`. The generated per-binding
getters (`env.some_secret() -> String`) are the same in all three.

### FormData / URLSearchParams

```rust
// A: test/src/form.rs:17-22, 35, 39-47
fn from_js(value: JsValue) -> Option<FormEntry> {                            // get: File | string | null -> JsValue
    if let Some(s) = value.as_string() { Some(FormEntry::Field(s)) }
    else { value.dyn_into::<File>().ok().map(FormEntry::File) }
}
if !form.has(NAME) { /* … */ }
form.get_all(NAME).into_iter().filter_map(FormEntry::from_js)
let q: Option<String> = params.get("q")?;                                    // URLSearchParams::get
```
```rust
// B: identical
```
```rust
// C: identical; the tagged-enum design covers primitive unions only, so `File | string` stays
// erased.
```
No difference. A handle-or-string union like this one would need ts-gen to emit an enum, and that
is independent of mono.

## 2. Port tally, A vs B

Regex counts over `git show <rev>:` of `ts-gen-worker/{src,test/src,benchmark/src}`, excluding the
generated `bindings.rs` and `env.rs` and skipping comment lines.

| pattern | A `104579e` | S15 `a806700` | B `2aa55cc` |
|---|---:|---:|---:|
| `JsString` (any mention) | 12 | 10 | 0 |
| `JsString::from` | 2 | 1 | 0 |
| `String::from(<js>)` | 12 | 11 | 6 |
| `.as_string()` | 14 | 14 | 9 |
| `unchecked_into` / `unchecked_ref` | 29 | 29 | 19 |
| `dyn_into` / `dyn_ref` | 7 | 7 | 7 |
| turbofish on bindings (`::<JsValue…>`) | 9 | 9 | 7 |
| all turbofish `::<` | 92 | 92 | 91 |
| `let x: T =` annotations | 86 | 89 | 81 |
| `None::<…>` | 0 | 0 | 0 |
| `&[] as &[…]` | 0 | 0 | 1 |
| `JsValue::from` / `from_str` / `from_f64` | 29 | 29 | 22 |
| `JsNullable::wrap` | 42 | 42 | 42 |
| owned list literals (`"x".to_string()]`) | 12 | 12 | 8 |
| helper detours (bulk-get helper, `for_each` + failure slot, `Array` entry parsing) | 3 | 3 | 0 |
| unstable-js-sys churn (`Reflect::get` → `Option`, `Promise::<JsValue>::new`) | 0 | 0 | 4 |

Lines changed A → B: 20 files, +114 −172. A → S15 was 16 files, +34 −36, mostly clippy borrow removal.
The remaining `String::from` (6) and `.as_string()` (9) are js-sys's own `JsString` returns
(`Error::name`, `Date::to_iso_string`) and `any` values (`MessageEvent::data`, attachments,
FormData). Ts-gen does not type those in any mode. The port has no `None::<&str>`, because it
never passes `None` to an `Option<impl JsStringLike>`. The email row in §1 shows what that call site looks like.

## 3. API diff

Generated `bindings.rs` (pre-`mono-patch` for the diffs, so stopgaps do not show), item-keyed by
`tools/api-diff`:

- **B vs S15**: 274 items changed, 0 added or removed, 5154 unchanged.
- **B vs A**: 1485 changed, 15 removed, 388 added, 3555 unchanged. S15 vs A was 1429 / 15 / 388.

B vs S15 by pattern (one item can carry several):

| pattern | items |
|---|---:|
| container `JsString` → `String` (`Map`, `Promise`, `Iterator`, `ArrayTuple`, `Object` for `Record`) | 98 |
| `JsOption` / `JsNullable` → `Option` inside containers and awaited generics | 60 |
| dictionary helper impls: `T` bound → `where for<'a> &'a T: IntoWasmAbi` | 51 |
| generic argument `T` → `&T` (pattern 6) | 41 |
| container `Boolean` → `bool` | 18 |
| constructor class parameters dropped or narrowed (pattern 7) | 16 |
| container `Number` → `f64` | 5 |

Type-level counts:

| pattern | A | S15 | B |
|---|---:|---:|---:|
| `JsString` (word) | 175 | 532 | 422 |
| `_js_string` twins | 1 | 384 | 384 |
| `Map<JsString, …>` | 44 | 44 | 0 |
| `Array<JsString>` (in B: twins only) | 3 | 21 | 18 |
| `Promise<JsString>` | 8 | 8 | 0 |
| `Object<JsString>` | 28 | 28 | 0 |
| `Iterator<JsString>` | 6 | 6 | 0 |
| `ArrayTuple<(JsString, …)>` | 4 | 4 | 0 |
| `JsNullable<JsString>` | 39 | 39 | 24 |
| `Function<fn(…JsString…)>` (callbacks) | 8 | 8 | 8 |
| `Boolean` / `Number` | 63 | 62 | 39 |
| `JsOption<` / `JsNullable<` | 246 | 246 | 186 |
| `impl JsStringLike` | 0 | 1268 | 1268 |
| `Option<impl JsStringLike>` | 0 | 7 | 7 |
| `&[impl JsStringLike]` | 0 | 117 | 117 |
| `impl js_sys::Js…Like` (primitive unions) | 0 | 13 | 13 |
| `Option<&str>` / `&[String]` | 7 / 117 | 0 / 0 | 0 / 0 |
| `: JsGeneric` bounds | 666 | 75 | 75 |

Every item that gets **worse in B than in S15**:
- **`Map<String, Option<String>>::get` (10 signatures)** cannot be called, because `Option<Option<String>>` has no ABI. In S15, `Map<JsString, JsNullable<JsString>>::get(&"k".into())` worked. The signatures are the `KVNamespace` bulk-`get` overloads, `get_text_with_slice`, and their `_promise` forms. Workaround: iterate (`into_iter().collect::<HashMap<_, _>>()`).
- **16 constructors lose class parameters.** S15 let you name them (`ReadableStream::<Uint8Array>::new()`) and required you to. B returns A's defaults, so a typed stream from a zero-argument constructor is `unchecked_into` again. Affected: `EventTarget::new`; `ReadableStream::new` and `new_with_underlying_byte_source{,_and_queuing_strategy}`; `ReadableStreamDefaultReader::new`; `Request::new{,_with_str,_with_url}` and `CfHostMetadata` on the `_and_init` forms; `TransformStream::new`; `WritableStream::new` and `new_with_underlying_sink{,_and_queuing_strategy}`; `WritableStreamDefaultWriter::new`.
- **`get_json<ExpectedValue>` (2 items)** now carries an `OptionFromWasmAbi` bound (stopgap #5 extended), because its awaited value is `Option<ExpectedValue>`.

Every item that is **still worse in B than in A** (S15 regressions B does not fix):
- **`Option<&str>` → `Option<impl JsStringLike>` (7):** passing `None` needs `None::<&str>`.
- **`&[String]` → `&[impl JsStringLike]` (117):** `&[]` needs `&[] as &[&str]`. In return, literal lists and `&Vec<String>` both work.
- **`&str` → `impl JsStringLike` (1268):**
  - Each Rust string type at a call site is its own import and shim (test worker: 472 imports in A, 488 in B; 177 generic shims).
  - Owned `String` shims free in a `try/finally`.
  - clippy removes `&` at 12 port sites.
- **`_js_string` twins (+383):** API surface only. None are used by the port. Stripping them leaves the output byte-identical and saves 0.9–1.0 s of compile time, a third of mono's overhead (§4).
- **Dictionary helpers and `&T` generics** need `where for<'a> &'a T: IntoWasmAbi` (51 helper items). Generic user code over these types must repeat that bound, where A needed `T: JsGeneric`.
- **`Map<_, Option<_>>::get`:** see above; in A it worked as `Map<JsString, JsNullable<JsString>>::get`.
- **Surface outside `bindings.rs` (unstable js-sys):**
  - `Reflect::get` returns `Result<Option<T>>`.
  - `Promise::new`'s resolver no longer infers as `Function` (`Promise::<JsValue>::new` plus `resolve.unchecked_ref()`, `test/src/util.rs:12-13`).
  - `Array::get` returns `Option<T>`.

## 4. Costs

**Compile errors and stopgaps.**
- `mono-patch` still applies the session-15 stopgaps unchanged:
  - #1: 33 extern fns
  - #1/#2: 16 impls
  - #3/#4: 2 twins removed, 2 renamed
  - #5: 2 `'static`
- One extension: #5 also adds `OptionFromWasmAbi` where an inline wrapper returns `Option<T>` (2 fns).
- `js_sys_unstable_apis` errors:
  - serde-wasm-bindgen: 6 errors, vendored (19 changed lines).
  - wasm-streams: 4 errors, vendored (10 lines).
  - Port: 3 errors in `ts-gen-worker` and 20 in the test crate.
  - `worker-sys`: 4 errors (`worker` was not attempted), so both are left out of B. The repo's `chomp test` (145) does not build under the cfg.
- With the feedback-8 fix, the port's turbofish is A's `get::<JsValue>(…)` again (S15 needed `::<JsValue, _>`).
- `Map::get` takes a `MapKey<K>` rather than `HashMap`'s `Borrow<Q>`, because the macro rejects `Q: ?Sized` ("unsupported in wasm-bindgen generics").

**Size.** Each state was built from its own commit with its own wasm-bindgen pin, on the same
machine:
- Benchmark worker: release build, `wasm-opt -Oz` with `benchmark/build.sh`'s flags.
- Test worker: debug build, covering the whole suite's surface.

A and S15 reproduce session 15's numbers exactly.

| | A | S15 | B | B − A |
|---|---:|---:|---:|---:|
| benchmark `index_bg.wasm` | 120,645 (51,600 gz) | 120,942 (51,576) | 121,251 (51,735) | +606 B (+0.5%), +135 gz |
| benchmark `index.js` | 33,871 (6,371 gz) | 35,064 (6,437) | 35,165 (6,449) | +1,294 B (+3.8%), +78 gz |
| benchmark wasm imports | 77 | 77 | 77 | 0 |
| test worker glue | 175,754 | 193,508 | 198,024 | +22,270 B (+12.7%) |
| test worker wasm imports | 472 | 478 | 488 | +16 |
| test worker generic shims (`// generic import`) | 0 | 157 | 177 | +177 |
| test worker `finally` blocks (owned-string frees) | 10 | 16 | 14 | +4 |

B built without the `_js_string` twins produces byte-identical benchmark and test outputs: no
call site uses a twin, so nothing of them survives into the output.

**Compile time** of the `ts-gen-worker` crate from clean (`cargo clean -p ts-gen-worker`, deps warm,
wasm32). The table gives the second of two runs. The first debug run of every state was cold (24–27 s) and is dropped; the release runs agree within 0.1 s.

| | A | S15 | B | B without twins |
|---|---:|---:|---:|---:|
| debug | 14.3 s | 17.1 s | 17.6 s | 16.6 s |
| release | 11.2 s | 13.4 s | 13.9 s | 13.1 s |
| test crate alone on top (debug) | 4.1 s | 4.2 s | 4.3 s | 4.3 s |

B − A is +3.3 s (+23%) debug and +2.7 s (+24%) release. The twins account for 1.0 s and 0.9 s of
that, so mono's own share is +16% / +17%. B is 0.5 s over S15, from the container leaves and the
feedback-8 wrappers. The test crate on its own is flat. Session 15's "test crate on top" figure
also rebuilt `ts-gen-worker`, so it is not comparable to this row.

## 5. "Fleshed out" checklist

| change | where | origin | size |
|---|---|---|---|
| `js_sys_unstable_apis` for wasm32 | `.cargo/config.toml` | built | 1 line |
| serde-wasm-bindgen 0.6.5 under the cfg (`get_unchecked`, `set_ref`, `Object::entries_typed`, `Array::from_iterable`) | `ts-gen-worker/vendor-patches/` + `[patch.crates-io]` | built | 19 lines |
| wasm-streams 0.7.0 under the cfg (`catch_map`, `get_unchecked`) | same | built | 10 lines |
| native leaves inside js-sys containers (`TypePosition::native_leaves`, `to_container`): `String`/`f64`/`bool`/`Option`/enums in `Promise`, `Map`, `Set`, iterators, `ArrayLike`, `Record`'s `Object`, nested `Array`, tuples. Not mono-specific: a non-mono build with unstable js-sys can use the same lowering (scratch-crate check, §5 end) | ts-gen `9e4636d` `typemap.rs` | built | ~70 lines |
| pattern 6: type-parameter arguments as `&T`; helper impls `where for<'a> &'a T: IntoWasmAbi` | ts-gen `9e4636d` `classes.rs` | built | ~30 lines |
| pattern 7: constructors bind only the class parameters their arguments mention, defaults otherwise (shared with the non-mono path) | ts-gen `9e4636d` `classes.rs` | built | net −20 lines |
| awaited mono `T \| undefined` as `Option<T>` (not `JsOption<T>`) | ts-gen `87882e1` | built | 5 lines |
| feedback 8: argument-position `impl Trait` kept in the public mono wrapper, forwarding to a hidden twin, so `get::<JsValue>("k")` has its declared arity | wasm-bindgen `4fe6881c0` `macro-support/codegen.rs` | built | ~90 lines |
| `ArrayTuple` getters mono; `first`/`last`/`into_tuple` for `FromWasmAbi` elements | wasm-bindgen `86a54f19d` js-sys | built | ~130 lines, mostly cfg-twinned getter declarations |
| `Map::get`/`has`/`delete` take `MapKey<K>` (`&K`, or `&str` for `String` keys) | same | built | ~25 lines |
| `Map::entries` unbounded; `Map` / `&Map` iterate as `(K, V)` (`MapEntries`) | same | built | ~40 lines |
| `Object::entries` → `Array<ArrayTuple<(String, T)>>` for any `T` | same | built | 3 lines |
| stopgaps #1–#5 for failures 1–5 (+ the `OptionFromWasmAbi` extension) | `ts-gen-worker/tools/mono-patch` | built (stopgap) | session 15 + ~15 lines |
| `T \| null` arguments as `Option<impl JsStringLike>` / `Option<f64>` / `Option<&T>` | ts-gen | assumed | small; the `Option<impl JsStringLike>` path exists for `T \| undefined` |
| constructor returns with the TS return type's concrete arguments (`ReadableStream<Uint8Array>`) | ts-gen IR (constructors carry no return type) | assumed | medium |
| `Option<Option<T>>` ABI, so `Map<_, Option<_>>::get` works | wasm-bindgen | assumed | medium; a nested-option encoding |
| typed `Promise::new` resolver under unstable | js-sys | assumed | small |
| one shim per JS function for owned and borrowed strings; no `// generic import` comment in release (feedback 9, 12) | wasm-bindgen CLI | assumed | medium |
| `worker` / `worker-sys` ported to unstable js-sys | `worker*` | assumed (excluded) | 4+ errors in `worker-sys` |
| callback parameters with Rust leaves (`Function<fn(String)>`, 8 sites) | js-sys `Function` + ts-gen | assumed | large (callbacks are erased by design) |

C would need the following instead (none built):
- #5167, with the root `&T` slot leak fixed.
- `JsStr<'a>` and a sealed `JsStringLike::as_js_str`, plus a list either type.
- Tagged enums with `From` impls for the 13 primitive unions.
- ts-gen lowering of container leaves to `String` and friends wherever the element is a bare `T` (works under #5167). Structural positions (`Option<T>`, `JsNullable<T>`, `Vec<T>`, closures) would keep handles.
- Hand-written `IntoJsGeneric`/`FromJsGeneric` for every user type used as a type argument (exported structs, ts-gen string enums: none get impls from the macro).

With `js_sys_unstable_apis` and the js-sys rows above added, C would also get B's container reads:
`Map::get("k")`, `(K, V)` iteration, and `Object::entries` with `String` keys. A scratch crate
compiled these against non-mono `extern` blocks returning `Map<String, Option<String>>`,
`Iterator<ArrayTuple<(String, String)>>`, `Promise<String>`, `Promise<Option<bool>>` and
`Option<Object<String>>`. It also confirmed that a leaf `T` on a ts-gen-style `T: JsGeneric`
getter still needs #5167. The ecosystem break comes with
that cfg in C exactly as in B.
