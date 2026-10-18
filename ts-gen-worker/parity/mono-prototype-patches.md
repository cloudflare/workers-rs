# Mono prototype: every patch

The phase-2 prototype builds `ts-gen-worker` on the full mono generics type system. Each patch
below is a prototype commit, not a refined PR: the "upstream" column says where it should land.
All branches are local and named `proto/mono`:

| repo | branch tip | base |
|---|---|---|
| workers-rs | `proto/mono` (workers-rs-3) | phase 1 `gbedford/tgw-mono-endstate` `2f32d52` |
| ts-gen | `proto/mono` (workers-rs-5) | D1's layer stack `0c813f2`…`0ec2bd9` + #122 (`6e22f0c`…`6d1dd4f`) |
| wasm-bindgen | `proto/mono` (workers-rs-4) | the submodule pin (#5390 + #5375) `ea363f37e` |

The workers-rs submodules point at the ts-gen and wasm-bindgen tips, so `proto/mono` in
workers-rs-3 builds the whole prototype. `js_sys_unstable_apis` is on for wasm32; `worker`,
`worker-sys`, `test/` and `examples/` are left out of the build (`-p` only).

## wasm-bindgen (macro, CLI, ABI)

| commit | files | what | why | upstream |
|---|---|---|---|---|
| `4fe6881c0` | `macro-support/src/codegen.rs` | mono wrappers keep argument-position `impl Trait` and forward to a hidden twin | feedback 8: `storage.get::<JsValue>("k")` has its declared arity | generic-mono PR |
| `160741bcf` | `src/convert/{traits,impls,slices}.rs`, `macro-support/src/{codegen,generics,parser}.rs`, `cli-support/src/{descriptor,descriptors,wit/mod}.rs`, ref test `generic-import-wire` | hidden `IntoWasmAbi::arg_into_abi`/`describe_arg`: `String`, `Option<String>`, `JsValue` and imported types cross borrowed through a slot dropped after the call | `&str`/`String`/`&String` (and `JsString`/`&JsString`) share one import; no owned-string `finally` frees (feedback 9) | generic-mono PR |
| `160741bcf` | same | CLI erases `NamedExternref` names before grouping mono imports | handle types that differ only in TS name share a shim | generic-mono PR |
| `160741bcf` | same | `// generic import` comments only under `--keep-debug` | feedback 12 (13 KB of debug glue) | generic-mono PR |
| `160741bcf` | same | `?Sized` mono type parameters | `Map::get(&Q) where K: Borrow<Q>` | generic-mono PR |
| `160741bcf` | same | direct bounds on a synthesized `impl Trait` param stay on the outer wrapper | `4fe6881c0` dropped them (`impl Clone` imports failed) | fold into the feedback-8 PR |
| `160741bcf` | same | `impl IntoWasmAbi for &Option<T>`; CLI reads `REF OPTIONAL T` as `OPTIONAL REF T` | `Map<_, Option<_>>::set(&k, &Some(v))` | wasm-bindgen |
| `cb872c471` | `macro-support/src/codegen.rs`, `src/convert/impls.rs`, `cli-support/src/descriptor.rs` | mono `Option<&T>` arguments (`for<'a> Option<&'a T>: IntoWasmAbi`), `Option<&prim>: IntoWasmAbi`; `unraw()` for derived idents | `T \| null` → `Option<&T>` for generic args; `__wbg_slot_r#type` panic | generic-mono PR |

## js-sys (all under `js_sys_unstable_apis` unless noted)

| commit | what | why | upstream |
|---|---|---|---|
| `86a54f19d` | `ArrayTuple` getters mono, `first`/`last`/`into_tuple` for `FromWasmAbi` elements; `Object::entries` → `Array<ArrayTuple<(String, T)>>` | Rust-typed tuple and record reads | js-sys unstable |
| `4aeed9f00` | `Map::{get,has,delete}(&Q) where K: Borrow<Q>` (replaces phase 1's `MapKey`); `Map::get` = `has` + `get_unchecked`, so it works for every `V` | `map.get("k")` like `HashMap`; `Map<String, Option<String>>::get` → `Option<Option<String>>` without a nested-option ABI | js-sys unstable |
| `83af8baa2` | `JsFunction` unbounded; `Function<fn(A..) -> R>` with any ABI `A`/`R` (incl. `()`); `Function::from_fn`; mono `call0..8` and `Function::call((&a, &b))`; mono `Promise::new` (typed resolver) | feedback 1 (type params inside `Function<…>`); callbacks with Rust leaves | js-sys unstable |
| `9d903011a` | small-string returns are `String` (`Error::{name,message}`, `Date::to_*string`, `Number::to_fixed`, `RegExp::{source,flags}`, `Function::name`); `Date::new_with_value(impl JsNumberOrStringLike)` | port's `String::from(js)` and `Date::new(&JsValue::from_f64(..))` | js-sys unstable |
| `366274d92` | `Iterator<T>` iterates as `T` (throw → `throw_val`; `try_iter` keeps the protocol's `Result`); `Entries<K, V>`; Map/Set `iter`/`IntoIterator`/`FromIterator`/`Extend`; Set lookups by `Borrow<Q>`; `Object::keys` → `Array<String>`; `JsStream<T: FromWasmAbi>` | Rust-collection behaviour; audit #3 (`HeadersIter`, `collect::<Result<…>>`) | js-sys unstable |
| `429a22215` | `From<ScopedClosure<'static, C>> for Function<F>` | `Closure` → typed `Function` without a cast | js-sys |
| `866b0d33d` | `Object<T>` as a record: `iter()`, `IntoIterator for &Object<T>`, `FromIterator`/`Extend<(K: PropertyKey, V: Borrow<T>)>`; `IntoIterator for &Array<T>` (both cfgs) | R2 custom-metadata helpers, `Reflect::set` loops | js-sys unstable |

## ts-gen

| commit | what | why | upstream |
|---|---|---|---|
| `f69a59d` | native leaves in js-sys containers (`Map<String, _>`, `Array<String>`, `Promise<String>`, `Set`, iterators, `Record`'s `Object`, tuples) in arguments, returns and fields; type-parameter arguments lent (`&T`); constructors bind only the class parameters their arguments mention | ideal end state (containers, pattern 6, pattern 7) | ts-gen mono PR |
| `ea1db10` | an awaited mono `T \| undefined` is `Option<T>` | `storage.get::<f64>(k)` → `Option<f64>` | ts-gen mono PR |
| `8a53f8c` | re-bless D1's two mono fixtures | rebase drift | folds into `f69a59d` |
| `cdf4f12` | `{@string copy\|ref\|both}` (member doc, `@returns`, declaration default, `--annotate`); twin named before keyword escaping (`type_js_string`) | Guy's string-return decision; feedback 10, 11 | ts-gen "string return modes" PR |
| `62843fb` | helper impl bounds from setter shapes (`&T`, `Option<T>`, `&[T]`) | feedback 2; replaces `mono-patch` #1/#2 | ts-gen mono PR (bounds) |
| `2744756` | `_typed` constructor twins bind every class parameter; concrete construct-signature returns (`ReadableStream<Uint8Array>` for a byte source) | ideal end state (constructors) | ts-gen mono PR (constructors) |
| `17df4e9` | `T \| null` arguments → `Option<impl JsStringLike>` / `Option<f64>` / `Option<&T>`; builder methods take an optional nullable value bare (`.signal(&s)`, `.prefix("p")`); an `ArrayBufferView` builder value upcasts into the setter | 42 `JsNullable::wrap` in the port | ts-gen mono PR (nullable args); Guy's call against #51's no-coercion rule |
| `1ed2145` | callbacks take native leaves, `void` is `()`, `void \| Promise<void>` is `Option<Promise<()>>`, `Promise<Undefined>` → `Promise<()>`; `() => Promise<T>` keeps `T`; helper bounds only on bare type-parameter mentions | audit #4; needs `83af8baa2` + `cb872c471` | ts-gen mono PR (bounds) |
| `64e785e` | an expanded null-admitting union takes bare arms plus a `_with_null` arm (`Response::new_with_null_and_init(&init)`, `R2Bucket::put_with_null(k)`) | audit #1(a): no `Some(..)` on every body | ts-gen mono PR (nullable args) |
| `ccd963b` | `Iterable<T>` arguments → slices; alias leaves native (`D1SessionBookmark = String`); undefined-only `any` returns → `Option<JsValue>` | audit #9, #10 | ts-gen mono PR (leaves) |

## workers-rs (`ts-gen-worker`)

| commit | files | what | why | upstream |
|---|---|---|---|---|
| `14be4c3` | `.cargo/config.toml` | `--cfg js_sys_unstable_apis` for wasm32 | the mono js-sys APIs live behind the cfg | workers-rs; drop once stable |
| `14be4c3` | `ts-gen-worker/vendor-patches/serde-wasm-bindgen`, `Cargo.toml` `[patch.crates-io]` | `get_unchecked`, `set_ref`, `Object::entries_typed`, `Array::from_iterable` (19 lines) | 0.6.5 has 6 errors under the cfg | serde-wasm-bindgen (cfg-gated) |
| `14be4c3` | `ts-gen-worker/vendor-patches/wasm-streams` | `catch_map`, `get_unchecked` (10 lines) | 0.7.0 has 4 errors under the cfg | wasm-streams |
| `df2ad74` | `ts-gen-worker/types/annotations.d.ts` | `{@string copy}` declaration defaults on 122 declarations, `{@string both}` on `Body`/`Blob`/`R2ObjectBody` `text()`; KV values keep both | small strings `String` only; twins where a body may go back to JS (389 → 76 twins) | workers-rs |
| `06ec97f` | `chompfile.toml`, `ts-gen-worker/tools` | `mono-patch` step and tool deleted | ts-gen output compiles as generated | workers-rs |
| `8ab0eab` | `ts-gen-worker/src/ext/websocket.rs` | `From<WebSocketPair> for (WebSocket, WebSocket)` | `_0()`/`_1()` getters from upstream's `0`/`1` keys | workers-rs |
| `8ab0eab` | `ts-gen-worker/{test,benchmark}/types/env.d.ts` | `readonly` + `{@string copy}` on `Env` | no setters or twins on bindings | workers-rs |
| `636cfab`…`507177f` | `ts-gen-worker/{src/ext,test/src,benchmark/src}` | port rewritten to each new surface | ideal idioms | workers-rs |

Superseded local fixes (folded upstream, not in the final tree): wasm-bindgen `a3d285150`
(raw-identifier slot names, now in `cb872c471`), ts-gen `3e42fce` (`ArrayBufferView` builder
upcast, now in `17df4e9`).
