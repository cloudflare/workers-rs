# Generic mono on the workers-rs port (session 15)

`ts-gen-worker` regenerated with `--experimental-generic-mono` from ts-gen `01ed819`
(`workers-mono-test`: the stack tip `45a278c` plus #122), compared with the non-mono baseline from
`45a278c` (session 15a). wasm-bindgen is `67e24940` (#5390) with #5375 on top. The signature-level
diff is `generic-mono-api-diff.md`; this file is the reading of it.

Verdict for workers-rs users, up front: the output does **not compile as emitted** (499 errors,
113 sites, five causes, none of them introduced by #122); with 55 stopgap edits it builds and the
whole suite passes unchanged. Once compiling, mono is mostly neutral-to-better for string-heavy code
and worse for generic types: constructors and generic-typed arguments now need annotations or
by-value moves that the erased API did not. #122 itself is clean: its share is the list patterns
(b.4, b.12 and 18 of b.5's twins), it caused no compile error, and its one ergonomic cost (`&[]`,
feedback 9) is the one `impl JsStringLike` already has for `None`.

## (a) What mono does

Every extern block becomes `#[wasm_bindgen(experimental_generic_mono)]`, so wasm-bindgen emits one
JS shim per Rust instantiation of a generic import instead of erasing type parameters to
`JsValue`; ts-gen accordingly drops the `JsGeneric` bound, passes generic-typed values by value,
and gives constructors the class's type parameters. String arguments become `impl JsStringLike`
(`&str`, `String`, `JsString`, … each crossing as itself), `number | bigint` and other primitive
unions become the matching `js_sys` `…Like` trait, and every string-returning member gains a
`_js_string` twin that returns the `JsString` without decoding it. #122 extends the string rule to
lists: `&[impl JsStringLike]` arguments and `Array<JsString>` twins of string-list getters.

## (b) Change patterns

Counts are items in Part 1 of the API diff (1,429 changed, 388 added, 15 removed of 5,055); an
item can show several patterns. Examples abbreviate paths (`impl JsStringLike` for
`impl ::wasm_bindgen::JsStringLike`, `Error` for `crate::Error`).

1. **`&str` argument → `impl JsStringLike`** — 853 items.
   ```rust
   pub fn set(this: &Headers, name: &str, value: &str) -> Result<(), JsException>;
   pub fn set(this: &Headers, name: impl JsStringLike, value: impl JsStringLike) -> Result<(), JsException>;
   pub async fn open(this: &CacheStorage, cache_name: &str) -> Result<Cache, Error>;
   pub async fn open(this: &CacheStorage, cache_name: impl JsStringLike) -> Result<Cache, Error>;
   ```
   **Neutral, slightly better.** A `JsString` in hand (from another binding) crosses without a
   UTF-8 round trip and an owned `String` needs no `&`; `&str` call sites are unchanged. Costs:
   `None` and `&[]` need annotations (feedback 9), turbofish arity changes (feedback 8), clippy's
   `needless_borrows_for_generic_args` now fires on `&s.to_string()` (12 port sites), and every
   distinct argument type is its own instantiation and JS shim (e).

2. **`JsGeneric` bound dropped** — 479 items (40 type declarations, the rest methods).
   ```rust
   pub type D1Result<T: JsGeneric = JsValue>;           pub type D1Result<T = JsValue>;
   pub fn props<Props: JsGeneric>(this: &ExecutionContext<Props>) -> Props;
   pub fn props<Props>(this: &ExecutionContext<Props>) -> Props;
   pub async fn json<T: JsGeneric + FromWasmAbi>(this: &Body) -> Result<T, Error>;
   pub async fn json<T>(this: &Body) -> Result<T, Error>;
   ```
   **Better in principle** (any ABI type, e.g. `Body::json::<String>()`, `D1Result<f64>`), **broken
   in practice** where the parameter reaches a non-mono generic such as `Function<fn(T)>`: 33 extern
   fns do not compile (feedback 1).

3. **Builder / factory `impl` bounds `JsGeneric` → `IntoWasmAbi`** — 51 impls.
   ```rust
   impl<Type: JsGeneric> KVNamespaceGetOptions<Type> { … }
   impl<Type: IntoWasmAbi> KVNamespaceGetOptions<Type> { … }
   ```
   **Neutral**, but under-bounded: 16 impls need more than `IntoWasmAbi` (feedback 1, 2).

4. **`&[String]` → `&[impl JsStringLike]`** (#122) — 117 items.
   ```rust
   pub async fn delete_with_slice(this: &R2Bucket, keys: &[String]) -> Result<(), Error>;
   pub async fn delete_with_slice(this: &R2Bucket, keys: &[impl JsStringLike]) -> Result<(), Error>;
   pub fn accept_web_socket_with_tags<Props>(…, tags: &[String]) -> Result<(), Error>;
   pub fn accept_web_socket_with_tags<Props>(…, tags: &[impl JsStringLike]) -> Result<(), Error>;
   ```
   **Better**: `&["a", "b"]` and `&Vec<String>` both work; no `Vec<String>` to build from
   literals. One cost: `&[]` needs a type (feedback 9).

5. **`_js_string` twins** — 385 added: 367 returning `JsString` (getters, async `text()`), 18
   returning `Array<JsString>` (#122).
   ```rust
   pub fn url<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>) -> String;
   pub fn url_js_string<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>) -> JsString;  // added
   pub fn get_all(this: &Headers, name: impl JsStringLike) -> Result<Vec<String>, JsException>;
   pub fn get_all_js_string(this: &Headers, name: impl JsStringLike) -> Result<Array<JsString>, JsException>;
   ```
   **Neutral to worse.** Useful when the value goes straight back to JS (no decode), but the
   surface grows by 385 methods for one representation choice, `Blob::type__js_string` shows the
   naming seam (feedback 10, 11), and none of the port needed one.

6. **Generic-typed argument `&T` → `T` by value** — 41 items.
   ```rust
   pub async fn send<Body: JsGeneric>(this: &Queue<Body>, message: &Body) -> Result<…>;
   pub async fn send<Body>(this: &Queue<Body>, message: Body) -> Result<…>;
   pub fn cf(self, val: &Cf) -> Self;                     pub fn cf(self, val: Cf) -> Self;
   pub async fn read_at_least<T: JsGeneric>(…, view: &T) -> …;   pub async fn read_at_least<T>(…, view: T) -> …;
   ```
   **Worse.** Callers must move or clone handles they used to lend, and for a type parameter not
   fixed by `this` a borrow still compiles but silently instantiates `T = &X` (c.3). #5390's own
   js-sys writes take `&T` for this reason (feedback 7).

7. **Constructors generic over the class's parameters** — 16 constructors.
   ```rust
   pub fn new() -> Result<TransformStream, Error>;
   pub fn new<I, O>() -> Result<TransformStream<I, O>, Error>;
   pub fn new_with_str(input: &str) -> Result<Request, JsException>;
   pub fn new_with_str<CfHostMetadata, Cf>(input: impl JsStringLike) -> Result<Request<CfHostMetadata, Cf>, JsException>;
   ```
   **Worse.** Function generics have no defaults, so `TransformStream::new()?` alone no longer
   infers; and the parameters are not tied to the arguments
   (`ReadableStreamDefaultReader::new<R>(stream: &ReadableStream)`,
   `ReadableStream::new_with_underlying_byte_source<R>` whose TS return is
   `ReadableStream<Uint8Array>`), so the annotation is an unchecked assertion (feedback 6).

8. **String type argument `JsString` → `String`** — 24 items.
   ```rust
   options: &KVNamespaceGetOptions<JsString>             options: &KVNamespaceGetOptions<String>
   Map<JsString, KVNamespaceGetWithMetadataResult<JsString, JsValue>>
   Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>
   ```
   **Neutral.** Consistent with mono's "Rust types cross as themselves"; `Map` keys stay
   `JsString` because `Map` is not a mono import in stable js-sys.

9. **`TypedArray` builder methods gain `where for<'a> &'a T: IntoWasmAbi`** — 22 items.
   ```rust
   pub fn sha256_with_typed_array<T: TypedArray>(self, val: &T) -> Self;
   pub fn sha256_with_typed_array<T: TypedArray>(self, val: &T) -> Self where for<'a> &'a T: IntoWasmAbi;
   ```
   **Neutral** (every js-sys typed array satisfies it); noise in docs.

10. **Primitive unions → `js_sys` `…Like` traits, collapsing overloads** — 12 changed, all 15
    removals (one is a rename, below).
    ```rust
    pub fn new(expected_length: f64) -> Result<FixedLengthStream, Error>;
    pub fn new_with_big_int(expected_length: &BigInt) -> Result<FixedLengthStream, Error>;  // removed
    pub fn new(expected_length: impl JsBigIntOrNumberLike) -> Result<FixedLengthStream, Error>;
    pub fn set_attribute(this: &Span, key: &str, value: bool) -> …;   // + _with_f64, _with_str (removed)
    pub fn set_attribute(this: &Span, key: impl JsStringLike, value: impl JsBooleanOrNumberOrStringLike) -> …;
    ```
    **Better**: one binding per JS function, `f64` call sites unchanged.
    `FixedLengthStream::new_with_f64_and_queuing_strategy` becomes `new_with_queuing_strategy`.

11. **`Option<&str>` → `Option<impl JsStringLike>`** — 7 items (`EmailAttachment` factories and
    `content_id`, `UnderlyingSource::set_type`). **Worse**: `None` no longer compiles without
    `None::<&str>` (feedback 9).

12. **`Vec<Boolean>` → `Vec<bool>`** (#122) — 1 item (`ValueKind::VecOfBool`). **Better.**

Unchanged: every type's derives and `extends`, enum shapes other than 12, all non-generic,
non-string signatures (3,611 items).

## (c) Interesting cases

1. **The output does not compile.** 499 errors at 113 sites from five causes (feedback 1–5); the
   same count and causes with or without #122 and with or without `js_sys_unstable_apis`. Most
   sites are in surface the port never calls (`ExportedHandler`, `UnderlyingSink`/`Source`,
   `Trace*`), reached through `--export` closure, so a crate that does not touch them still cannot
   build.
2. **`impl Trait` changes turbofish arity.** The macro rewrites `key: impl JsStringLike` to a named
   parameter, so
   ```rust
   storage.get::<JsValue>("count")          // E0107: method takes 2 generic arguments
   storage.get::<JsValue, _>("count")
   ```
   Plain Rust accepts the first form (explicit generics with argument-position `impl Trait`,
   stable since 1.63).
3. **A borrow silently becomes the type argument.**
   ```rust
   let init = RequestInit::builder().cf(&cf);          // RequestInitBuilder<&IncomingRequestCfProperties>
   Request::new_with_str_and_init(&uri, &init.build())?   // Request<_, &IncomingRequestCfProperties>
   ```
   The error surfaces at the `?` far from the cause (`expected Request, found Request<_, &…>`).
4. **Typed streams by annotation.** The port's `TransformStream::new()?` became
   ```rust
   let transform: TransformStream<Uint8Array, Uint8Array> = TransformStream::new()?;
   writer.write_with_chunk(chunk).await        // was write_with_chunk(&chunk)
   ```
   which is a real improvement in what the writer accepts, but nothing checks the annotation
   against JS.
5. **`Queue<JsValue>` and untyped writers want `JsValue` by value.**
   `writer.write_with_chunk(&Uint8Array::from(data))` → `write_with_chunk(Uint8Array::from(data).into())`;
   `queue.send(&body)` → `queue.send(body)`. The erased API took `&JsValue` and deref-coerced.
6. **`type__js_string`, `use__js_string`** (34 methods): the `_js_string` suffix is appended to the
   escaped name.
7. **Twins collide or name erased parameters.** The literal-keyed inline wrapper `KVNamespace::
   get_text` and its twin are both named `get_text` (E0592); `KVNamespace::list_js_string`
   returns `KVNamespaceListResult<JsValue, JsString>` for a one-parameter type (E0107) because
   `Key extends string` was erased from the type but not from the twin's substitution.
8. **Owned strings cost a heavier shim.** clippy steers `URL::new(&self.url())` to
   `URL::new(self.url())`; the `String` instantiation's shim frees the buffer in a
   `try/finally` with the call guard, where the `&str` one is a one-liner. Same JS function, two
   shims if both forms are used.
9. **One JS function, several shims.** The debug test worker has 157 generic shims (imports 472 →
   478). Where call sites pass different Rust string types to one binding, each instantiation is its
   own import: `Headers.set`, `WebSocket.close`, `WebSocket.send` and `set_type` each have a
   `Ref(String)` and an owned `String` shim. Every shim also carries a `// generic import …`
   comment: 13,350 B of the debug glue (feedback 12).
10. **`js_sys_unstable_apis` is not needed and cannot be enabled.** Nothing in the output requires
    it (same errors with and without it, and the patched output builds both ways). Enabling it
    breaks crates.io `serde-wasm-bindgen 0.6.5` (6 errors) and `wasm-streams 0.7.0` (4 errors),
    both in `worker`'s and `ts-gen-worker`'s graphs, and a cfg is global to the build so it cannot
    be scoped to the ts-gen-worker crates. The cfg-only behaviour #122's integration tests add
    (`Array<String>` passed to an `ArrayLike<string>` parameter) is therefore not exercised here.
11. **The five compile failures split by origin.** On plain main `f8466d4`, 1 and 2b reproduce;
    2a, 3, 4, 5 do not, because they need stack layers: `T | undefined` → `Option<T>` (`2d6cfaf`),
    primitive-constraint erasure (`592db90`), literal-keyed inline wrappers (`49afaaf`).

## (d) Port call-site changes

Required by mono (16 sites):

| File | Before | After | Cause |
|---|---|---|---|
| `ext/kv.rs` | `KVNamespaceGetOptions::builder(&JsString::from(value_type))` → `…<JsString>` | `KVNamespaceGetOptions::builder(value_type.to_string())` → `…<String>` | b.6, b.8 |
| `ext/serde.rs` | `self.get::<JsValue>(key)` | `self.get::<JsValue, _>(key)` | feedback 8 |
| `ext/interop.rs` | `init.cf(&cf)` | `init.cf(cf)` | b.6, c.3 |
| `test/src/cache.rs`, `request.rs` ×2 | `let transform = TransformStream::new()?;` | `let transform: TransformStream<Uint8Array, Uint8Array> = TransformStream::new()?;` | b.7 |
| same three | `writer.write_with_chunk(&chunk)` / `(&Uint8Array::from(&buf[..]))` | `write_with_chunk(chunk)` / `(Uint8Array::from(&buf[..]))` | b.6 |
| `test/src/r2.rs`, `send_email.rs`, `socket.rs` | `write_with_chunk(&Uint8Array::from(…))` | `write_with_chunk(Uint8Array::from(…).into())` | c.5 |
| `test/src/socket.rs` | `write_with_chunk(&chunk.value().unwrap_or_default())` | `write_with_chunk(chunk.value().unwrap_or_default())` | b.6 |
| `test/src/queue.rs` | `my_queue.send(&body)` | `my_queue.send(body)` | b.6 |
| `test/src/d1.rs` | `stmt.first_with_col_name::<JsValue>(col)` | `stmt.first_with_col_name::<JsValue, _>(col)` | feedback 8 |
| `test/src/ext.rs` | `storage.get::<JsValue>("count")` | `storage.get::<JsValue, _>("count")` | feedback 8 |

Required by clippy `needless_borrows_for_generic_args` once arguments are `impl JsStringLike`
(12 sites, all `&expr` → `expr`): `ext/cors.rs` ×5 (`headers.set(…, &join(…))`),
`ext/http.rs` (`URL::new(&self.url())`), `test/src/lib.rs` (`headers.set("x-sum",
&sum.to_string())`), `counter.rs` (`ws.send_with_str(&format!(…))`), `container.rs` ×3
(`&message_text(&msg)…`, `&reason.reason()`), `send_email.rs` (`&scenario.raw()`).

No change in `benchmark/`, the macros, or any spec. `test/src/env.rs` and `benchmark/src/env.rs`
are regenerated with the flag (their string setters take `impl JsStringLike`, getters gain twins).

## (e) Size and compile time

Benchmark worker (release, `wasm-opt -Oz`, against `9439547`; session 15a's non-mono rebuild
had the same sizes):

| | non-mono | mono | Δ |
|---|---|---|---|
| `index_bg.wasm` | 120,645 B (51,600 gz) | 120,942 B (51,576 gz) | +297 B (+0.2%), −24 B gz |
| `index.js` glue | 33,871 B (6,371 gz) | 35,064 B (6,437 gz) | +1,193 B (+3.5%), +66 B gz |
| wasm imports | 77 | 77 | 0 |

Test worker (debug, the whole suite's surface): glue 175,754 → 193,508 B (+17,754 B, +10.1%), of
which 13,350 B is the per-shim `// generic import` comment; imports 472 → 478 (c.9); owned-string
shims with a deferred free 1 → 7 (c.8).

Compile time of the `ts-gen-worker` crate from clean (warm deps, two runs each, this machine; one
cold baseline run of 25.3 s dropped): debug 14.4 s → 17.3–18.4 s (+20–28%), release 11.2–11.4 s →
13.5–14.5 s (+18–29%); the test crate on top 15.4–15.6 s → 18.3–19.0 s (+18–23%).

## (f) Feedback for the generic-mono work

Origin: **main** reproduces on ts-gen `f8466d4`; **stack** needs a layer of `45a278c`; **#122**
introduced by it; **wasm-bindgen** is not a ts-gen issue. 1–5 are the stopgap edits `mono-patch`
applies (`ts-gen-worker/tools/`, run by `chomp build:ts-gen-worker:types`, each site marked
`// ts-gen mono: … (generic-mono.md #N)`).

1. **Type parameters inside `Function<…>` are unbounded** (main; 33 extern fns, part of 16 impls).
   ```ts
   interface Handler<Env = unknown> { fetch?: (env: Env) => void; }
   declare function serve(h: Handler): void;
   ```
   Actual: `pub fn fetch<Env>(this: &Handler<Env>) -> Option<Function<fn(Env) -> Undefined>>;`,
   E0277 `Env: ErasableGeneric / UpcastFrom<Env> / JsCast` (js-sys `Function<T: JsFunction>`
   needs `JsGeneric` arguments, mono or not). Expected: compiles. Proposal: keep `: JsGeneric` on
   a type parameter wherever it occurs inside a non-mono generic (`Function`, and generated types
   used as `Function` arguments, `Request<CfHostMetadata>` here), on the fn and on builder impls
   that call it; the stopgap does exactly that.

2. **Builder / factory impls bound only `IntoWasmAbi`** (16 impls with 1).
   ```ts
   interface Rows<T> { rows: T[]; }          // main
   interface Slot<T> { value: T | undefined; }   // stack (2d6cfaf makes it Option<T>)
   declare function f(r: Rows<string>, s: Slot<string>): void;
   ```
   Actual: `impl<T: IntoWasmAbi> Rows<T> { pub fn new(rows: &[T]) … }` → E0277 `T:
   VectorRefIntoWasmAbi`; `impl<T: IntoWasmAbi> Slot<T> { pub fn new(value: Option<T>) … }` →
   E0277 `T: OptionIntoWasmAbi`. Proposal: derive the impl bounds from the setters' argument
   shapes (`&[T]` → `VectorRefIntoWasmAbi`, `Option<T>` → `OptionIntoWasmAbi`), or emit
   `where &'a [T]: IntoWasmAbi`-style clauses per method.

3. **`_js_string` return twin passes an argument for an erased parameter** (stack: `592db90`).
   ```ts
   interface Entry<M, Key extends string = string> { name: Key; meta?: M; }
   type ListResult<M, Key extends string = string> =
     | { done: false; keys: Entry<M, Key>[]; cursor: string }
     | { done: true; keys: Entry<M, Key>[] };
   interface Store<Key extends string = string> { list<M = unknown>(): Promise<ListResult<M, Key>>; }
   declare function store(): Store;
   ```
   Actual: `pub type ListResult<Meta = JsValue>;` and `list_js_string(this: &Store) ->
   Result<ListResult<JsValue, JsString>, JsValue>` (E0107). Expected: no twin (with `Key` erased
   it equals `list`). Proposal: compute twins after primitive-constraint erasure, and skip a twin
   whose substituted signature equals its base.

4. **The `_js_string` twin of a literal-keyed inline wrapper takes the base name** (stack:
   `49afaaf`).
   ```ts
   interface Store {
     get(key: string, type: "text"): Promise<string | null>;
     get<T = unknown>(key: string, type: "json"): Promise<T | null>;
   }
   declare function store(): Store;
   ```
   Actual: two `pub async fn get_text(&self, key: impl JsStringLike)` (E0592), one returning
   `Option<String>`, one `Option<JsString>` (its extern is correctly `get_text_js_string_js`).
   Expected: `get_text_js_string`. Proposal: name the wrapper from the extern's suffix.

5. **Inline wrapper type parameters lack the bounds the mono extern needs** (stack: `49afaaf`).
   Same input as 4. Actual: `pub async fn get_json<T>(&self, key: impl JsStringLike) -> … {
   self.get_json_js(key, "json").await }` → E0310 / E0311 `T` may not live long enough. Proposal:
   copy the extern's implied bounds (`T: 'static` for async; generally whatever the macro adds) onto
   the wrapper.

6. **Constructors take the class's type parameters with nothing to infer them from** (main;
   16 constructors). Ergonomic, compiles.
   ```ts
   declare class Stream<R = any> { constructor(); write(chunk: R): void; }
   ```
   Actual: `pub fn new<R>() -> Result<Stream<R>, JsValue>;` — `Stream::new()?` is E0282 unless
   the binding is annotated; `ReadableStreamDefaultReader::new<R>(stream: &ReadableStream)` cannot
   be inferred even from its argument. Expected: `Stream::new()` gives `Stream` (the default), as in
   non-mono. Proposal: a constructor keeps only the class parameters its arguments mention (the
   rule `8f9b687` applies to methods), the rest take the declared defaults; a TS return with
   concrete arguments (`ReadableStream<Uint8Array>`) is returned as written.

7. **Generic-typed arguments by value** (main; 41 items).
   ```ts
   interface Queue<Body = unknown> { send(message: Body): Promise<void>; }
   interface Init<Cf = unknown> { cf?: Cf; }
   ```
   Actual: `send<Body>(this: &Queue<Body>, message: Body)`, builder `cf(self, val: Cf)`. Callers
   move or clone handles they lent before, and where the parameter is free a borrow compiles as
   `Cf = &X` (c.3). Expected: borrowing works as in non-mono. Proposal: take `&T` with `for<'a> &'a
   T: IntoWasmAbi`, as #5390's js-sys element writes do, so `&x` keeps `T = X` and deref coercion
   still applies.

8. **Argument-position `impl Trait` breaks turbofish** (wasm-bindgen).
   ```rust
   #[wasm_bindgen(experimental_generic_mono)]
   extern "C" { pub type S; #[wasm_bindgen(method)] pub fn get<T>(this: &S, key: impl JsStringLike) -> Option<T>; }
   s.get::<JsValue>("k");   // E0107: method takes 2 generic arguments, `T`, `__WasmBindgenImplTrait0`
   ```
   Expected: compiles, as for a plain Rust fn. Proposal: keep `impl Trait` in the emitted wrapper's
   signature and desugar only inside it.

9. **`None` and `&[]` need a type** (main for `Option<impl JsStringLike>`; #122 for `&[impl
   JsStringLike]`).
   ```ts
   interface Attachment { contentId?: string | undefined; }
   declare function attach(a: Attachment): void;
   declare function tag(ws: object, tags: string[]): void;
   ```
   Actual: `set_content_id(None)` and `tag(&o, &[])` are E0283; callers write `None::<&str>`,
   `&[] as &[&str]`. Expected: both compile. Proposal: optional string setters keep `Option<&str>`
   (the `JsString` case is rare there); for lists, document the `&[] as &[&str]` idiom in
   CONVENTIONS, since `impl Trait` cannot carry a default.

10. **`type__js_string`** (main; 34 methods). The twin suffix is appended to the keyword-escaped
    name (`type_`, `use_`). Proposal: append to the unescaped name (`type_js_string`).

11. **`_js_string` twin volume** (main, +18 from #122; 385 methods). Every string getter doubles.
    Proposal: emit twins only under an opt-in (or only for members marked hot), since `String`
    returns stay the primary API and the port used none.

12. **Every generic shim carries a comment, and string types split shims** (wasm-bindgen).
    Debug test worker: 157 shims, 13,350 B of `// generic import …` comments (75% of the +17.8 KB
    glue growth); `Headers.set("k", "v")` and `Headers.set("k", s.to_string())` are two imports. Proposal: emit the comment only under `--keep-debug`; lower an
    owned `String` passed to `impl JsStringLike` through the borrowed ABI so both share one shim.
