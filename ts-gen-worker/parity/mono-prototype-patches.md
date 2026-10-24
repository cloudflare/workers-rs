# Mono prototype: every patch

The prototype builds `ts-gen-worker` on the full mono generics type system. It was built in two
phases: phase 1, an earlier mono build whose generated bindings needed hand-applied fixes, and
phase 2, which replaced those fixes with ts-gen and wasm-bindgen changes and added the rest. Both
phases' patches are listed. Each is a prototype commit, not a refined PR: the "upstream" column
says where it should land.

| repo | branch | base |
|---|---|---|
| workers-rs | this branch | `main` |
| ts-gen | `gbedford/mono-prototype`, tip `3393ade` | main `b0af305` → option-null-reads → #52 → #53 → #54 → #66 → #67 → #68 → #69 → the mono commits (`ts-gen-prs.md` "Restack 4") |
| wasm-bindgen | `gbedford/mono-prototype`, tip `3a62aa7d4` | main `39ae823bb` → #5428 → #5427 → #5375 → #5390 → #5425 → #5426 → 20 residual commits (see "After the session-16 restack") |

ts-gen and wasm-bindgen commits are cited by subject, with an id (`tgN`, `wbN`, numbered in
branch order; a ts-gen fixture re-bless has no row) that the PR grouping below uses. The
workers-rs submodules pin both tips, so this branch builds the whole prototype.
`js_sys_unstable_apis` is set only for the ts-gen-worker builds (`chompfile.toml`), so `worker`
and the rest of the workspace build without it, against the same wasm-bindgen.

## wasm-bindgen (macro, CLI, ABI)

| id | commit | files | what | why | upstream |
|---|---|---|---|---|---|
| wb2 | `Keep argument-position impl Trait in generic-mono wrapper signatures (scratch)` | `macro-support/src/codegen.rs` | mono wrappers keep argument-position `impl Trait` and forward to a hidden twin | `storage.get::<JsValue>("k")` has its declared arity | generic-mono PR |
| wb3 | `Generic mono: one binding per wire format, ?Sized params, &Option<T> (proto)` | `src/convert/{traits,impls,slices}.rs`, `macro-support/src/{codegen,generics,parser}.rs`, `cli-support/src/{descriptor,descriptors,wit/mod}.rs`, ref test `generic-import-wire` | hidden `IntoWasmAbi::arg_into_abi`/`describe_arg`: `String`, `Option<String>`, `JsValue` and imported types cross borrowed through a slot dropped after the call | `&str`/`String`/`&String` (and `JsString`/`&JsString`) share one import; no owned-string `finally` frees | generic-mono PR |
| wb3 | same | same | CLI erases `NamedExternref` names before grouping mono imports | handle types that differ only in TS name share a shim | generic-mono PR |
| wb3 | same | same | `// generic import` comments only under `--keep-debug` | 13 KB of debug-only glue in release builds | generic-mono PR |
| wb3 | same | same | `?Sized` mono type parameters | `Map::get(&Q) where K: Borrow<Q>` | generic-mono PR |
| wb3 | same | same | direct bounds on a synthesized `impl Trait` param stay on the outer wrapper | wb2 dropped them (`impl Clone` imports failed) | fold into the generic-mono PR (with wb2) |
| wb3 | same | same | `impl IntoWasmAbi for &Option<T>`; CLI reads `REF OPTIONAL T` as `OPTIONAL REF T` | `Map<_, Option<_>>::set(&k, &Some(v))` | wasm-bindgen (the `&Option<T>` blanket was replaced by `RefIntoWasmAbi for Option<T>` in wb22) |
| wb6 | `Generic mono: Option<&T> arguments, raw identifiers (proto)` | `macro-support/src/codegen.rs`, `src/convert/impls.rs`, `cli-support/src/descriptor.rs` | mono `Option<&T>` arguments (`for<'a> Option<&'a T>: IntoWasmAbi`), `Option<&prim>: IntoWasmAbi`; `unraw()` for derived idents | `T \| null` → `Option<&T>` for generic args; `__wbg_slot_r#type` panic | generic-mono PR |
| wb11 | `Generic mono: extends with type arguments, Vec<Option<T>> returns (proto)` | `macro-support/src/codegen.rs`, `src/convert/slices.rs`, 2 reference files | `extends = Parent<A, B>` with concrete type arguments (parent conversions via `<Parent as JsCast>::unchecked_from_js`); `Vec<Option<T>>` returns for handles, `String`, `bool`, `char`, numbers | `IdentityTransformStream extends TransformStream<…, Uint8Array>`; `TraceEventInfo::traces -> Vec<Option<String>>` | wasm-bindgen |
| wb13 | `Generic mono: extends naming the child's params, one &T ABI impl for every handle, ArrayBuffer bytes (proto)` | `macro-support/src/codegen.rs`, `src/convert/impls.rs`, `src/lib.rs` (`JsGeneric: WasmDescribe`) | `extends = Par<T, T>` naming the child's parameters; one blanket `IntoWasmAbi for &T` / `Option<&T>` over every `T: JsGeneric` | ts-gen keeps `Child<T>` arguments; drops the `for<'a> &'a T: IntoWasmAbi` clause on handle-bounded helpers | wasm-bindgen (the `&T` / `Option<&T>` blanket was removed in wb22) |
| wb15 | `macro (proto): catch on a Promise<T> return rejects on a sync throw` | `macro-support/src/{ast,parser,codegen}.rs`, `src/convert/traits.rs`, js-sys | `catch` on an import whose return is `Promise<..>` (not `Result`) returns a synchronous throw as a rejected promise (`FromCaughtException`, `Promise::reject_typed`) | a sync throw from a promise-returning member must surface at `.await`, not unwind through wasm | wasm-bindgen |
| wb16 | `macro (proto): slice modes on generic mono args apply per instantiation` | `macro-support/src/codegen.rs`, js-sys | `slice = "copy"` (and the other slice modes) on a generic mono argument applies per instantiation: `&[u8]` crosses as a fresh `Uint8Array`, handles as themselves | a stored `BufferSourceLike` must not be a view into wasm memory | wasm-bindgen |
| wb18, wb19, wb20 | `Require explicit ABI bounds on experimental_generic_mono imports`; `Place mono where-clause suggestions after the signature, review fixes`; `Place mono where-clause suggestions after the declared return type` | `macro-support/src/{codegen,generics}.rs`, ui tests, guide | the mono wrapper gets no synthesized ABI bounds: the declaration's bounds are the whole contract, and a missing one is a rustc error with a `where` suggestion placed after the return type | rustdoc showed `impl JsStringLike + IntoWasmAbi + WasmDescribe` and `for<'__wbg> &'__wbg X: IntoWasmAbi + WasmDescribe` on every signature | wasm-bindgen#5385 |
| wb21 | `Generic mono (proto): RefIntoWasmAbi, declared bounds read inline, explicit bounds across js-sys` | `src/convert/{traits,impls}.rs`, `macro-support/src/{codegen,generics}.rs`, js-sys | `RefIntoWasmAbi` (first cut, a blanket over `&T: IntoWasmAbi`), so a `&T` argument is bounded `T: RefIntoWasmAbi` instead of an HRTB; simple `P: Bounds` render inline on the wrapper and on the hoisted `impl` header; js-sys declares its bounds explicitly | aesthetic signatures: `new_with_str_and_init(input: impl JsStringLike, init: &RequestInit<Cf>) -> Result<Request<JsValue, Cf>, JsException>` with no `where` | fold into wasm-bindgen#5385 |
| wb22 | `Generic mono (proto): per-type RefIntoWasmAbi, no blanket &T ABI impl` | `src/convert/{traits,impls}.rs`, `macro-support/src/codegen.rs`, js-sys | per-type `RefIntoWasmAbi`, mirroring `RefFromWasmAbi` (own `Abi`, `ref_into_abi`, and an `Option<&Self>` ABI): the macro implements it per imported type beside the restored upstream `IntoWasmAbi for &Foo<..>`, wasm-bindgen for `JsValue`, `JsError`, strings and primitives; a mono `&T` / `Option<&T>` argument lowers through `T::ref_into_abi`. The `&T` and `&Option<T>` blankets are gone; js-sys has no HRTB left | no blanket impls. Of the prototype's earlier blankets, two structural `Option<T>` impls stay, `RefIntoWasmAbi for Option<T>` and `VectorFromWasmAbi for Option<T: JsGeneric>`, because a per-type impl for `Option<Foo>` breaks the orphan rule | fold into wasm-bindgen#5385 |
| wb23 | `Generic mono (proto): JsNullable<T> for any ABI type T` | `src/sys.rs`, js-sys tests | mono `JsNullable<T>` for any ABI `T` (native leaves: `JsNullable<String>`, `JsNullable<f64>`, `JsNullable<bool>`): reads `into_option()`, `is_null()`, `Option::from`; writes `JsNullable::wrap`, `JsNullable::new()`, `Some(v).into()`; `Debug`, `PartialEq` (`===`). Per-type impls, no blanket. `Option<JsValue>` returns needed no change: they already fold `null` | null model: a nullable type argument (KV bulk get's `Map<String, string \| null>`) keeps `null` distinct | wasm-bindgen (with the generic-mono PR) |
| wb24 | `Generic mono (proto): Vec of dynamic unions, &[impl BufferSourceLike]` | `macro-support/src/codegen.rs`, `src/convert/slices.rs`, js-sys | a dynamic-union enum works as a `Vec` element in both directions (per-enum `WasmDescribeVector`, `VectorFromWasmAbi`, `VectorIntoWasmAbi`, each element dispatched like a single value); `BufferSourceLike: VectorRefIntoWasmAbi`, so `&[impl BufferSourceLike]` is a written slice whose `&[u8]` / `Vec<u8>` elements cross as `Uint8Array` copies | `FormData::get_all` and buffer lists (`File` bits) stayed `JsValue` | wasm-bindgen (with the generic-mono PR) |

## js-sys (all under `js_sys_unstable_apis` unless noted)

| id | commit | what | why | upstream |
|---|---|---|---|---|
| wb1 | `js-sys (scratch): Rust-typed element access under js_sys_unstable_apis` | `ArrayTuple` getters mono, `first`/`last`/`into_tuple` for `FromWasmAbi` elements; `Object::entries` → `Array<ArrayTuple<(String, T)>>` | Rust-typed tuple and record reads | js-sys unstable |
| wb4 | `js-sys (proto): Map keys by Borrow<Q>; Map::get for any V` | `Map::{get,has,delete}(&Q) where K: Borrow<Q>` (replaces phase 1's `MapKey`); `Map::get` = `has` + `get_unchecked`, so it works for every `V` | `map.get("k")` like `HashMap`; `Map<String, Option<String>>::get` → `Option<Option<String>>` without a nested-option ABI | js-sys unstable |
| wb5 | `js-sys (proto): Function<fn(A..) -> R> with Rust argument and return types` | `JsFunction` unbounded; `Function<fn(A..) -> R>` with any ABI `A`/`R` (incl. `()`); `Function::from_fn`; mono `call0..8` and `Function::call((&a, &b))`; mono `Promise::new` (typed resolver) | type parameters inside `Function<…>`; callbacks with Rust leaves | js-sys unstable |
| wb7 | `js-sys (proto): String returns for small strings, Date::new_with_value` | small-string returns are `String` (`Error::{name,message}`, `Date::to_*string`, `Number::to_fixed`, `RegExp::{source,flags}`, `Function::name`); `Date::new_with_value(impl JsNumberOrStringLike)` | port's `String::from(js)` and `Date::new(&JsValue::from_f64(..))` | js-sys unstable |
| wb8 | `js-sys (proto): Rust iteration for Iterator, Set, Map; FromIterator/Extend; JsStream of any T` | `Iterator<T>` iterates as `T` (throw → `throw_val`; `try_iter` keeps the protocol's `Result`); `Entries<K, V>`; Map/Set `iter`/`IntoIterator`/`FromIterator`/`Extend`; Set lookups by `Borrow<Q>`; `Object::keys` → `Array<String>`; `JsStream<T: FromWasmAbi>` | Rust-collection behaviour: no hand-written `HeadersIter`, no `collect::<Result<…>>` | js-sys unstable |
| wb9 | `js-sys (proto): Function from Closure; Promise<()> and generic Function ABI tests` | `From<ScopedClosure<'static, C>> for Function<F>` | `Closure` → typed `Function` without a cast | js-sys |
| wb10 | `js-sys (proto): Object<T> records as Rust maps; IntoIterator for &Array` | `Object<T>` as a record: `iter()`, `IntoIterator for &Object<T>`, `FromIterator`/`Extend<(K: PropertyKey, V: Borrow<T>)>`; `IntoIterator for &Array<T>` (both cfgs) | R2 custom-metadata helpers, `Reflect::set` loops | js-sys unstable |
| wb12 | `js-sys (proto): String returns for JSON::stringify and Intl` | `JSON::stringify*` → `Result<Option<String>, JsValue>` (`None` for `undefined`); 28 `Intl` string returns are `String` | built-in string returns read as `String`, not `JsString` | js-sys unstable |
| wb13 | `Generic mono: extends naming the child's params, one &T ABI impl for every handle, ArrayBuffer bytes (proto)` | `ArrayBuffer::to_vec`, `From<&[u8]>`/`From<Vec<u8>>` for `ArrayBuffer`, `From<ArrayBuffer> for Vec<u8>` | `Uint8Array` round trips for bytes | js-sys unstable |
| wb14 | `js-sys (proto): BufferSourceLike, Function::once` | sealed `BufferSourceLike` for `&[u8]` (wasm-memory view), `Vec<u8>` (copy), `ArrayBuffer`, `DataView` and every typed array, owned and `&`; `Function::once(f)` inferred like `from_fn` | one argument for `ArrayBuffer \| ArrayBufferView`; no `block_concurrency_while` turbofish | js-sys unstable |
| wb17 | `js-sys (proto): Promise<T, E> with the await error type E` | `Promise<T, E = JsValue>`: `IntoFuture<Output = Result<T, E>>` for `E: From<JsValue>`, `E` erased for the ABI; `upcast()` changes `E`; `JsFuture<T, E>` | async members can await to the crate's error type | js-sys unstable |
| wb25 | `Generic mono (proto): js-sys ArrayHeadRest<H, T> for TS [H, ...T[]]` | `ArrayHeadRest<H = JsValue, T = JsValue>`, TS `[H, ...T[]]`: `head() -> H`, `rest() -> Array<T>` (a `slice(1)` copy), `into_parts()` / `Into<(H, Array<T>)>` | D1 `raw({ columnNames: true })` returns column names then rows; with no head/rest type it was `Vec<JsValue>` | js-sys unstable |

## ts-gen

| id | commit | what | why | upstream |
|---|---|---|---|---|
| tg1 | `Mono end state (scratch): native container leaves, lent generics, constructor defaults` | native leaves in js-sys containers (`Map<String, _>`, `Array<String>`, `Promise<String>`, `Set`, iterators, `Record`'s `Object`, tuples) in arguments, returns and fields; type-parameter arguments lent (`&T`); constructors bind only the class parameters their arguments mention | containers had JS leaves (`Map<JsString, _>`, `Array<JsString>`), generic arguments were taken by value, and constructors bound parameters nothing fixed | ts-gen mono PR |
| tg2 | `Mono end state (scratch): awaited generic Option/Vec keep Rust form` | an awaited mono `T \| undefined` is `Option<T>` | `storage.get::<f64>(k)` → `Option<f64>` | ts-gen mono PR |
| tg3 | `{@string copy\|ref\|both} picks a mono string return's forms; twin named before keyword escaping` | `{@string copy\|ref\|both}` (member doc, `@returns`, declaration default, `--annotate`); twin named before keyword escaping (`type_js_string`) | a string return reads as `String`; the `JsString` twin only where asked for | ts-gen "string return modes" PR |
| tg4 | `Bound per-mono helper impls by their setter shapes and callback parameters by JsGeneric` | helper impl bounds from setter shapes (`&T`, `Option<T>`, `&[T]`) | replaces phase 1's hand-applied `mono-patch` #1/#2 | ts-gen mono PR (bounds) |
| tg5 | `Per-mono constructors: a _typed twin binds every class parameter; concrete construct returns as written` | `_typed` constructor twins bind every class parameter; concrete construct-signature returns (`ReadableStream<Uint8Array>` for a byte source) | a typed instance from a zero-argument constructor needed a cast | ts-gen mono PR (constructors) |
| tg6 | `Per-mono null-admitting arguments are Option; builder methods take an optional field's value bare` | `T \| null` arguments → `Option<impl JsStringLike>` / `Option<f64>` / `Option<&T>`; builder methods take an optional nullable value bare (`.signal(&s)`, `.prefix("p")`); an `ArrayBufferView` builder value upcasts into the setter | 42 `JsNullable::wrap` in the port | ts-gen mono PR (nullable args); superseded for writes by the null model (tg38), which keeps ts-gen#51's no-coercion rule |
| tg7 | `Per-mono callbacks take native leaves and void as (); bound helpers only on bare type parameters` | callbacks take native leaves, `void` is `()`, `void \| Promise<void>` is `Option<Promise<()>>`, `Promise<Undefined>` → `Promise<()>`; `() => Promise<T>` keeps `T`; helper bounds only on bare type-parameter mentions | callbacks were typed over erased leaves; needs wb5 + wb6 | ts-gen mono PR (bounds) |
| tg8 | `Per-mono expanded null-admitting unions take bare arms and a null arm of their own` | an expanded null-admitting union takes bare arms plus a `_with_null` arm (`Response::new_with_null_and_init(&init)`, `R2Bucket::put_with_null(k)`) | no `Some(..)` on every body | ts-gen mono PR (nullable args) |
| tg9 | `Per-mono Iterable arguments as slices, native alias leaves, optional any as Option<JsValue>` | `Iterable<T>` arguments → slices; alias leaves native (`D1SessionBookmark = String`); undefined-only `any` returns → `Option<JsValue>` | `Iterable<T>` arguments and alias leaves were erased | ts-gen mono PR (leaves) |
| tg10 | `--annotate: @retype replaces a member's type; side namespaces reach namespace members` | `--annotate`: `@retype` (the side member's type replaces a getter's or every overload's return); side `declare namespace` reaches namespace members | `Body::body` is a byte stream; TailStream's 64 twins | ts-gen "annotate: retype + namespaces" PR |
| tg11 | `Per-mono overloads keyed by a literal type argument are bindings of their own` | overloads keyed by a literal type argument (`get(key, "text" \| { type: "text" })`) are bindings of their own: `get_text_with_options`, `get_json_with_options::<T>`, `get_with_metadata_array_buffer_with_options`, … | KV's typed reads collapsed to one text-typed form | ts-gen mono PR (overloads) |
| tg12 | `Per-mono IntoIterator for &Type when a declaration has [Symbol.iterator]()` | `IntoIterator for &T` from `[Symbol.iterator]()` (`Headers`, `FormData`, `URLSearchParams`, `SqlStorageCursor<T>`) | iterating `Headers` and `FormData` needed a hand-written iterator | ts-gen mono PR (iteration) |
| tg13 | `Per-mono overload suffixes imply the primary arm and name dictionary arms by parameter` | overload suffixes imply the primary arm and name dictionary arms by parameter (`fetch_with_init`, `set_timeout_with_ms_delay`, `put_with_options`); names over 50 chars 97 → 40 | overload and inline-type names over 50 characters | ts-gen mono PR (names) |
| tg14 | `Per-mono inline parameter types are named <Parent><Method><Param>` | inline parameter types named `<Parent><Method><Param>` under mono (`R2BucketGetOptions`, not `R2BucketOptions2`) | overload and inline-type names over 50 characters | ts-gen mono PR (names) |
| tg15 | `Per-mono _typed twins for methods whose nested return-only type parameters erase` | `_typed::<T>` twins for methods whose nested return-only type parameters erase (`getWithMetadata<Metadata>`, `list<Metadata>`, D1 `all`/`raw`/`run`/`batch`, DO `get` by keys) | return-only type parameters: a whole-return `T` keeps the generic; a nested one gets a twin (§7 of the report) | ts-gen mono PR (return-only params) |
| tg16 | `Per-mono returned lists read nullable elements as Option` | returned lists read nullable elements as `Option` (`Vec<Option<String>>`) | nullable list elements were erased; needs wb11 | ts-gen mono PR (leaves) |
| tg17 | `Per-mono bare generic-class arguments take any instantiation; extends keeps concrete arguments` | a bare generic-class argument at its `any` defaults takes any instantiation (`&ReadableStream<impl Sized>`); `_typed` reader/writer constructors unify with the stream; `extends` keeps concrete arguments | a generic-class argument at its defaults needed a cast | ts-gen mono PR (generic args) |
| tg18 | `Per-mono dictionary suffixes don't depend on overload order; drop the trailing null arm` | dictionary suffixes independent of overload order (R2 `get_with_options`); no trailing `null` arm duplicating `new()` | names depended on overload order; a `null` arm duplicated `new()` | ts-gen mono PR (names) |
| tg19 | `Per-mono Partial<X> parameters name a synthesized all-optional XPartial; undefined type arguments are ()` | a `Partial<X>` parameter names a synthesized all-optional `XPartial` (`KVNamespaceGetOptionsPartial`); `undefined` type arguments and defaults are `()` (`DurableObjectStub<T = ()>`) | `Partial<X>` arguments and `undefined` type arguments were erased | ts-gen mono PR (types) |
| tg20 | `Per-mono literal type-argument bindings set the field the literal types` | literal type-argument bindings are wrappers that set the field the literal types (`get_text_with_options` sets `type: "text"`) | the return type followed the method, the runtime result the caller's `type` | ts-gen mono PR (overloads) |
| tg21 | `Per-mono extends keeps the child's own parameters; typed-array helpers drop the &T bound` | `extends` keeps the child's own parameters (`Child<T> extends Par<T, T>`); helpers over a `TypedArray`/handle-bounded `T` drop the `for<'a> &'a T: IntoWasmAbi` clause | helper `for<'a>` clauses on handle-bounded `T`; needs wb13 | ts-gen mono PR (bounds) |
| tg22 | `Per-mono optional properties typed only never or undefined have no accessors` | optional properties typed only `never` or `undefined` get no accessors (D1's `error?: never`) | `Option<Undefined>` getters | ts-gen mono PR (types) |
| tg23 | `Per-mono dictionaries of literals are built by the binding` | a dictionary argument whose fields are all literals is set by the binding: `raw_with_column_names{,_typed}()`, `get_reader_byob()`; a `{ x?: false }` arm folds into the omitted form | D1 `raw`'s tuple return trusted the caller's `columnNames` | ts-gen mono PR (overloads) |
| tg24 | `Per-mono Partial getters are optional, field-literal wrappers use the setter, this returns the receiver` | `Partial` getters return `Option`; field-literal wrappers call the typed setter instead of `Reflect::set`; `this` returns lower to the receiver (`Span::set_attribute`); names (`json_with_init`, `with_session_with_constraint`, `verify`/`measure` arms) | `Partial` getters, field-literal setters and `this` returns needed casts | ts-gen mono PR (overloads, names) |
| tg25 | `Per-mono ArrayBuffer \| ArrayBufferView arguments take impl BufferSourceLike` | `ArrayBuffer \| ArrayBufferView` and `BufferSource` lower to `impl js_sys::BufferSourceLike` (`Option<…>` with `null`); one `_buffer_source` binding replaces the `_array_buffer` + `_typed_array` arms | one argument per buffer source instead of an arm per type; needs wb14 | ts-gen mono PR (types) |
| tg26 | `Per-mono async members return their Promise<T>` | async members are one import returning `Promise<T>` (`T` the awaited form), with no `async fn` / `_promise` pair (293 twins → 0) | one value that awaits and passes to JS | ts-gen mono PR (async) |
| tg27 | `Per-mono M[K] over a concrete M binds once per key` | `M[K]` over a concrete `M` binds once per key, with the key's literal set by the binding (`WebSocket::add_event_listener_message(&Function<fn(MessageEvent)>)`), declared on subclasses for the first parent's methods; overloads told apart by one literal are named by it | event listeners took `fn(JsValue)` and a cast | ts-gen mono PR (types) |
| tg28 | `Per-mono Promise<T> imports catch a synchronous throw` | `Promise<T>` imports keep `catch`, so a synchronous throw is a rejection at `.await` | a synchronous throw must surface at `.await`; needs wb15 | ts-gen mono PR (async) |
| tg29 | `Per-mono setters copy a &[u8] buffer source` | setters whose value is `impl BufferSourceLike` carry `slice = "copy"`; builders and factories store through them (15 setters) | a dictionary outlives the `&[u8]` borrow; needs wb16 | ts-gen mono PR (types) |
| tg30 | `Per-mono buffer sources copy unless {@slice view}` | buffer-source arguments copy a `&[u8]` unless `{@slice view}`; `{@slice handle}` takes handles only; `--annotate` carries `@param` `{@slice}` tags | a view is only safe where the callee copies before returning | ts-gen mono PR (types) |
| tg31 | `A tuple with a rest element lowers to an array` | `[A, ...B[]]` lowers to an array of the common element type (ungated) | D1 `raw`'s header-and-rows tuple was typed as a 2-tuple | ts-gen |
| tg32 | `Per-mono types reached only by an omitted argument aren't emitted` | inline types reached only by an omitted argument are not emitted | orphan `D1PreparedStatementRawOptions2` | ts-gen |
| tg33 | `Per-key listeners take the options and bool arms` | per-key listeners gain `_with_options` / `_with_bool` and the listener-object pairs (16 → 48 on `WebSocket`) | `{ once }` / `{ signal }` needed the erased form | ts-gen mono PR (types) |
| tg34 | `Per-mono buffer-source modes go on the argument` | buffer-source slice modes go on each argument, not the fn | a `{@slice view}` on one argument no longer depends on the others | ts-gen mono PR (types) |
| tg35 | `Per-mono async members return Promise<T, E>` | async members return `Promise<T, E>` with `E` the `--fallible` error type (`crate::Error`) | tails await to the crate's error again; needs wb17 | ts-gen mono PR (async) |
| tg36 | `Per-mono nullable union returns are Option of their enum` | a nullable handle-or-primitive union return is `Option` of its generated enum (`FormData::get` → `Option<GetReturnKind>`) | `File \| string \| null` was `JsValue` | ts-gen mono PR (types) |
| tg37 | `Per-mono imports declare their ABI bounds inline` | new pass `codegen/abi_bounds.rs` states each per-mono import's bounds inline: `T` argument `IntoWasmAbi`, `&T` / `Option<&T>` `RefIntoWasmAbi`, `&[T]` `VectorRefIntoWasmAbi`, return `FromWasmAbi`; `Option<T>` / `Vec<T>` returns are bounded whole in a first-order `where` (`Option<T>: FromWasmAbi`); nested parameters, concrete types and `impl …Like` get nothing. A builder's bound on its own parameter is method-level. Non-mono output unchanged | wasm-bindgen#5385: the declaration is the whole contract; the 12 helper `for<'__wbg>` clauses go | ts-gen mono PR after #5385 |
| tg38 | `Per-mono, null is an ordinary arm of a written argument` | a written `T \| null` (argument, setter, builder value, required field) is an ordinary union arm: the bare value plus a `_with_null` form (`clear_timeout_with_null()`, `set_prefix_with_null()`, `.prefix_with_null()`); an optional trailing `null` arm stays omitted; `any \| null` is `&JsValue` | null model: `None` sent `undefined` where TS said `null` (ts-gen#51) | ts-gen mono PR (null model) |
| tg39 | `Per-mono, a type only ever read has no write side` | a type only ever read (return, getter, awaited value) has no write side: no constructor, setters or builder (37 types, e.g. `KVNamespaceListResult`, `R2Objects`, Queue metrics) | null model: output-only dictionaries had setters nobody can use | ts-gen mono PR (null model) |
| tg40 | `Per-mono, a nullable type argument is JsNullable` | a nullable type argument is `JsNullable<T>` with the native leaf (`Map<String, JsNullable<String>>`), `T \| undefined` is `Option<T>`; reads of `any \| null` are `Option<JsValue>`; returned `Vec<T \| null>` stays `Vec<Option<T>>` | null model: `Map<String, Option<String>>` sent and read `null` as `undefined` | ts-gen mono PR (null model) |
| tg41 | `annotate: a side property with no setter makes the input's read-only` | `--annotate`: a side property declared `readonly` (or a lone `get x()`) drops the input's setter, with its `_with_null` form | workers-types drops `readonly` on JSG instance properties, so 31 setters threw a `TypeError` | ts-gen "annotate: retype + namespaces" PR |
| tg42 | `Per-mono, output-only reaches parents and open-record intersections` | output-only reaches a parent reached only through output-only children (`D1Response`) and the named member of an open-record intersection, which also types the field (`meta: D1Meta & Record<string, unknown>` → `meta() -> D1Meta`) | `D1Result` exposed `set_meta`/`set_success` through `Deref`; `D1Meta` had a dead write side and `meta()` was `JsValue` | ts-gen mono PR (null model) |
| tg43 | `Per-mono, a branch factory takes its required field bare` | a discriminated-union branch factory takes a field its branch requires bare (`EmailAttachment::new_inline(content_id: impl JsStringLike, …)`), not the merged `string \| undefined` | the `inline` branch requires `contentId`, yet took `Option` and `None::<&str>` | ts-gen mono PR (factories) |
| tg44 | `Per-mono, Promise-typed getters return Promise<T, E>` | Promise-typed getters return `Promise<T, E>` like methods (`socket.opened()`, `closed()`, `writer.ready()`, `digest()`, `exit_code()`) | a tail `.await` of one rejected with `JsValue` | ts-gen mono PR (async) |
| tg45 | `Per-mono, a union enum is shared only by sites of one JS name` | a union enum is shared only by sites of one JS name, and named from it (`ssec_key() -> Option<SsecKeyKind>`, `sha1() -> Option<Sha1Kind>`, SubtleCrypto `iv()` → `IvKind`); a name already taken is qualified by its class (`HibernatableWebSocketEventInfoInfoKind`) | shape-shared enums took the first site's name: R2's `ssec_key()` returned `MessageEventInitDataKind` | ts-gen mono PR (types) |
| tg46 | `Per-mono, lists of unions keep their element types` | lists of unions keep their element types: a returned `(A \| B)[]` is `Vec<…Kind>` (`FormData::get_all -> Vec<GetAllReturnKind>`, `R2ListOptions::include -> Option<Vec<IncludeKind>>`); a written one expands per element arm, named for the element (`indexes_with_str_slice`, `Blob::new_with_buffer_source_slice`, `to_with_email_address_slice`), then `_with_js_value_slice(&[JsValue])`; `Array<BufferSource>` is `&[impl BufferSourceLike]` | `get_all` and written union lists were `JsValue`; needs wb24 | ts-gen mono PR (leaves) |
| tg47 | `Per-mono, iterable items and typed-array type parameters carry their own bounds` | a generic iterable's `IntoIterator` bounds the item (`impl<'a, T: FromWasmAbi> IntoIterator for &'a SqlStorageCursor<T>`); a method type parameter constrained to typed arrays is `T: TypedArray + …` (`Crypto::get_random_values`, BYOB `read`/`read_at_least`) | the impl delegated through `where Iterator<T>: IntoIterator`, and the TS `extends` constraint was dropped | ts-gen mono PR (bounds) |
| tg48 | `Per-mono, a head-and-rest tuple is ArrayHeadRest<H, R>` | `[H, ...T[]]` with `H` and `T` different is `ArrayHeadRest<H', T'>` (D1 `raw_with_column_names()`, `raw_with_column_names_typed::<T>()`); a common element type stays a list; non-mono WAT comments print the declared tuple (ungated, `.rs` output unchanged) | D1 `raw`'s column names and rows were one `Vec<JsValue>`; needs wb25 | ts-gen mono PR (leaves) |
| tg49 | `Per-mono, a field's literal choice within one shape is a factory argument` | a required field offering several literals within one shape is a factory argument, typed as its setter (`IncomingRequestCfProperties::new(.., edge_request_keep_alive_status: f64, ..)`, `IncomingRequestCfPropertiesTLSClientAuth::new(.., cert_revoked: CertRevokedKind, ..)`); 48 factories → 7 | each literal was a numbered factory (`new_0`…`new_5`, `builder_0`…`builder_5`) | ts-gen mono PR (factories) |

## workers-rs (`ts-gen-worker`)

| files | what | why | upstream |
|---|---|---|---|
| `chompfile.toml`, `benchmark/build.sh`, the three crate roots | `--cfg js_sys_unstable_apis` for the ts-gen-worker builds only (`RUSTFLAGS`, own `target/mono`); without it the crates are empty on the host and a `compile_error!` on wasm32 | the mono js-sys APIs live behind the cfg, which `worker` does not build under | workers-rs; drop once stable |
| `ts-gen-worker/vendor-patches/serde-wasm-bindgen`, `Cargo.toml` `[patch.crates-io]` | `get_unchecked`, `set_ref`, `Object::entries_typed`, `Array::from_iterable` (19 lines) | 0.6.5 has 6 errors under the cfg | serde-wasm-bindgen (cfg-gated) |
| `ts-gen-worker/vendor-patches/wasm-streams` | `catch_map`, `get_unchecked` (10 lines) | 0.7.0 has 4 errors under the cfg | wasm-streams |
| `ts-gen-worker/types/annotations.d.ts` | `{@string copy}` declaration defaults on 122 declarations, `{@string both}` on `Body`/`Blob`/`R2ObjectBody` `text()`; KV values keep both | small strings `String` only; twins where a body may go back to JS (386 → 73 twins in `bindings.rs`) | workers-rs |
| `ts-gen-worker/src/ext/websocket.rs` | `From<WebSocketPair> for (WebSocket, WebSocket)` | `_0()`/`_1()` getters from upstream's `0`/`1` keys | workers-rs |
| `ts-gen-worker/{test,benchmark}/types/env.d.ts` | `readonly` + `{@string copy}` on `Env` | no setters or twins on bindings | workers-rs |
| `ts-gen-worker/src/ext/serde.rs` | `DurableObjectStorage::put_entries_serialized` (a struct's fields as DO entries) | `put_with_entries::<JsValue>(&to_value(..)?.unchecked_into())` at call sites | workers-rs |
| `ts-gen-worker/types/annotations.d.ts` | `{@slice view}` on `Response`'s body, KV/R2 `put`, R2 `uploadPart`, SubtleCrypto data arguments; `{@slice handle}` on `respondWithNewView` | a `&[u8]` view only where the callee copies before returning | workers-rs |
| `ts-gen-worker/types/annotations.d.ts` | `@retype` `Body::body`, `R2ObjectBody::body`, `Socket::{readable,writable}` to `Uint8Array` streams; `{@string copy}` on the TailStream namespace | upstream types them `ReadableStream` (`any` chunks); 64 twins | workers-rs (or workers-types) |
| `ts-gen-worker/types/annotations.d.ts` | `@retype` `MessageEvent.data` to `string \| ArrayBuffer \| Blob \| any`: a `DataKind` enum whose `JsValue` arm catches a `MessagePort` structured clone | `data` was `any`; `Blob` from the 2026-03-17 compat date, and an unmatched union value throws in the getter | workers-rs |
| `ts-gen-worker/{src/ext,test/src}` | null-model port: setters and builders take the value bare (`set_prefix(p)`, `set_metadata(&v)`, `clear_timeout(id)`); KV bulk reads `JsNullable<String>` (`into_option()`); the KV metadata `null` filter and the `ResponseInit` headers `null` guard removed | `Option<JsValue>` reads always folded `null`, so both were dead code | workers-rs |
| `ts-gen-worker/types/annotations.d.ts` | `readonly` on the properties workerd defines without a setter: `Request` (10), `Response` (9), `WebSocket` `readyState`/`url`/`protocol`/`extensions`, `WebSocketPair` `0`/`1`, `SqlStorageCursor.columnNames` | their setters threw a `TypeError`; `binaryType` stays writable | workers-rs (upstream: workers-types) |
| `ts-gen-worker/{src/ext,test/src,benchmark/src}` | the ext code and the ported test suite and benchmark, rewritten for each binding change | idiomatic use of each new surface | workers-rs |

## Upstream PR grouping

Each row of the tables above is one change. Landing them is a smaller set of PRs:

| PR | commits | depends on |
|---|---|---|
| wasm-bindgen: generic mono (arity, one import per wire format, `?Sized`, `Option<&T>`, raw idents) | wb2, wb3, wb6 | — |
| wasm-bindgen: `extends = Parent<A, B>`, `Vec<Option<T>>` returns | wb11, part of wb13 | — |
| js-sys unstable: Rust-typed collections (tuples, `Map`/`Set`/`Object`, iteration) | wb1, wb4, wb8, wb10 | — |
| js-sys unstable: `Function<fn(A..) -> R>`, `from_fn`, `once`, typed `Promise::new` | wb5, part of wb14 (+ wb9, stable) | — |
| js-sys unstable: `BufferSourceLike` | part of wb14 | — |
| js-sys unstable: `Promise<T, E>` | wb17 | — |
| js-sys unstable: `ArrayHeadRest<H, T>` | wb25 | — |
| wasm-bindgen: slice modes on generic mono arguments | wb16 | — |
| js-sys unstable: `String` returns, `Date::new_with_value`, `ArrayBuffer` bytes | wb7, wb12, part of wb13 | — |
| ts-gen: string return modes | tg3 | — |
| ts-gen: `--annotate` `@retype` + namespaces, `readonly` | tg10, tg41 | — |
| ts-gen mono: leaves (containers, awaited `Option`, `Iterable` slices, aliases, nullable list elements, lists of unions, head-and-rest tuples) | tg1, tg2, tg9, tg16, tg46, tg48 | wb11, wb24, wb25 |
| ts-gen mono: bounds and callbacks | tg4, tg7, tg21, tg47 | wb5, wb6, wb13 |
| ts-gen mono: constructors and return-only parameters (`_typed`) | tg5, tg15 | — |
| ts-gen mono: nullable arguments | tg6, tg8 | wb6 |
| ts-gen mono: overloads and names | tg11, tg13, tg14, tg18, tg20, tg23, tg24 | — |
| ts-gen mono: generic-class arguments, `Partial`, `()` | tg17, tg19, tg22 | wb11 |
| ts-gen mono: buffer-source arguments | tg25, tg29, tg30, tg34 | wb14, wb16 |
| wasm-bindgen: `catch` on `Promise<T>` returns | wb15 | — |
| wasm-bindgen#5385: explicit mono ABI bounds, plus per-type `RefIntoWasmAbi` | wb18, wb19, wb20, wb21, wb22 | — |
| wasm-bindgen: mono `JsNullable<T>` for any ABI `T` | wb23 | generic-mono PR |
| wasm-bindgen: `Vec` of dynamic unions, `&[impl BufferSourceLike]` | wb24 | generic-mono PR |
| ts-gen mono: per-key `M[K]` bindings | tg27, tg33 | — |
| ts-gen: rest tuples, orphan inline types | tg31, tg32 | — |
| ts-gen mono: nullable union-enum returns, per-name enums | tg36, tg45 | — |
| ts-gen mono: `Promise<T, E>` async returns | tg26, tg28, tg35, tg44 | wb15, wb17 |
| ts-gen mono: generated `IntoIterator` | tg12 | wb10 |
| ts-gen mono: explicit ABI bounds | tg37 | wasm-bindgen#5385 |
| ts-gen mono: factories (a branch's required field bare, a field's literal choice as an argument) | tg43, tg49 | — |
| ts-gen mono: null model (null arms, output-only types, `JsNullable<T>` type arguments) | tg38, tg39, tg40, tg42 | wb23 |
| serde-wasm-bindgen, wasm-streams: build under `js_sys_unstable_apis` | `ts-gen-worker/vendor-patches` | js-sys unstable |
| workers-rs: inputs (`annotations.d.ts`, `env.d.ts`), ext, port, `MIGRATING.md` | workers-rs rows | all of the above |

## After the session-16 restack

wasm-bindgen main `39ae823bb` has #5384 (the slice/sequence series), #5385 (wb18–wb20), #5410 /
#5420 / #5423 (strict `Option<T>` and `Nullable<T>`, `MaybeIntoWasmAbi`), #5388 and #5406. The
stack above it, bottom to top: #5428 (2: `Option<T>` reads `null` as `None` again, the strict
descriptor tag removed), #5427 (4: `Option<T>` / `Nullable<T>` as sequence elements and `Function`
signature slots, js-sys element methods per-mono under `js_sys_unstable_apis`), #5375 (3), #5390
(18, wb1 superseded by its element-API series, `RefIntoWasmAbi` sealed with a blanket over
`&T: IntoWasmAbi`, new `MonoGeneric`), #5425 (`JsCanon`, 2), #5426 (wb17 `Promise<T, E>`, 3), and
20 residual commits: wb2, wb3, wb4, wb5, wb6, wb7, wb8, wb9, wb10, wb11, wb12 (split in two: the
second adapts main's `raw_json` test), wb13 (without the blanket `&T` ABI impl, which main's
`RefIntoWasmAbi` and #5390's `JsGeneric: MonoGeneric` cover), wb14, wb15, wb16, wb22 (tests only:
the per-type `RefIntoWasmAbi` redesign is replaced by main's sealed blanket; wb21 dropped for the
same reason), wb23, wb24, wb25.

What #5427 and #5428 took over (the kept commits lose the hunk, their messages say so): #5427
already declares 38 of the methods #5390 makes mono (`Array::{set,fill,includes,push,pop,splice,
with,at,…}`, `Map::{get,set,has,delete,get_or_insert}`, `Set::{add,has,delete}`,
`Function::{call0-9,bind1-9,apply}`, `resolve`) with the same `RefIntoWasmAbi` contract, so
#5390's "Function call/bind/apply and the JsFunction traits mono" shrinks from 915 to 150
insertions: #5427's `JsFunction*` / `JsArgs` / `FunctionIntoClosure` have unbounded signature
slots, so #5390's `MonoGeneric`-bounded cfg-split traits are dropped (its tests and docs stay).
#5375's approved schema hash is recomputed over #5428's removed descriptor tag. In the residual:
wb6 "Option<&T> arguments" keeps only the `OptionIntoWasmAbi for &Option<T>` half (now on
#5427's `MaybeRefIntoWasmAbi`) since `Descriptor::option` no longer takes `strict`; wb11's
`Vec<Option<T>>` slice impls are dropped for #5427's `optional_scalar_vectors!` /
`optional_handle_vectors!`; the `Borrow<Q>` keyed `Map` / `Set` reads supersede #5427's `&K` ones.
No prototype test asserted the strict `null` read; #5428's `option_null` tests pass unchanged.

Rebase-forced changes kept from the session-14 pass: `&Option<T>` crosses through
`T: RefIntoWasmAbi` by one generic impl; `()` gains `IntoWasmAbi` for `&()` so it is
`MonoGeneric`; js-sys collection/iteration helpers declare `RefIntoWasmAbi` / `MonoGeneric`
bounds for #5390's bounded element writes; `FromCaughtException` is implemented for
`Promise<T, E>`.

Verified on the tip `3a62aa7d4` (`cargo +stable` / `+1.88`: the workspace's 1.91 pin leaks into
the submodule and changes symbol mangling): root wasm tests 590/0, js-sys 820/0 stable and 919/0
under `js_sys_unstable_apis`, macro ui tests on 1.88, CLI reference tests 84/84 with `BLESS=1`
changing nothing, schema hash tests green; the five `termination*` / `memory64_threads` node
tests fail identically on main here. The intermediate layer tips were compile-checked only where
a fix was needed; each PR's own reference snapshots may need a re-bless against #5427's shim
hashes before it is pushed.

Open against this wasm-bindgen: `Map<K, V>` iteration bounds `V: MonoGeneric` (a residual
prototype impl; a read needs only `FromWasmAbi`), and `&Nullable<T>` has no `IntoWasmAbi`, so a
`Map<String, Nullable<String>>` is walked as `JsValue`s in `ext/kv.rs`; `f64` / `i64` are not
`OptionFromWasmAbi` (their `Option` has an ABI of its own), which ts-gen now sidesteps by bounding
a generic `Option<T>` read whole (`Option<T>: FromWasmAbi`); a `#[wasm_bindgen]` enum has no
`&Enum` ABI (`IntoWasmAbi` by value only, unlike the primitives' `impl IntoWasmAbi for &T`), so an
enum as a generic class's type argument cannot be written through `T: RefIntoWasmAbi`
(`Holder::<Mode>::new(&Mode::Slow)` in ts-gen's `enum_positions_mono` test).
