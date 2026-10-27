# ts-gen PR plan: `agent/pr-audit` (43 commits on `e139daa` / main)

One PR per commit, bottom-up, each PR based on the branch of the one below. Titles are the
commit subjects; the description is the commit body (already in the merged-PR register: the
problem, what changes, what the workers-types snapshot diff shows). Each entry gives the
dependency that forces the order, and a risk note for the reviewer.

Audit state (session 6): every commit builds, passes `just test`, `just clippy`,
`just fmt-check` and `cargo check --target wasm32-unknown-unknown` of the integration crate on
its own; `workers-types.rs` is blessed at every commit; the final tree equals the pre-audit
`gbedford/workers-parity` tip apart from two doc additions (README `--export` roots example,
CONVENTIONS line for the overload-group merge). Rewrites against the previous branch: `b2d9e5c`
squashed into `--error-enum`, `5b9ba39` into "each overload's own return type", `ad5ee90` into
"inherited members"; `2d63fac` split into the string-enum fix and the ABV builder-bound fix;
every commit touching the workers-types snapshot now ends its message with a one-sentence
account of that diff.

**Independent** = cherry-picks onto main without conflict and passes alone, so it can be posted
out of order. Everything else conflicts textually with an earlier commit in the same file (the
dependency column names the real one; others are line-adjacency only).

| # | SHA | Title | Depends on | Independent |
|---|---|---|---|---|
| 1 | `484dfe9` | Add `--error-type <path>` for a configurable default error type | – | yes |
| 2 | `cffe4c7` | Add `--fallible` policy: one Result binding per sync callable | 1 (`GenerateOptions` plumbing) | no |
| 3 | `53af3c1` | Add `--error-enum`: `@throws` types become one generated enum | 1, 2 | no |
| 4 | `9a89359` | String enums: the `Variant = "value"` shape, passed by value | – (snapshot adjacency only) | yes |
| 5 | `8a456e1` | Builder methods for ArrayBufferView fields carry the setter's TypedArray bound | 4 (shares the `enum-args` fixture) | no |
| 6 | `a0c925d` | Bind renamed function exports to their public name | – | yes |
| 7 | `17fd715` | Emit `js_name` for accessors whose key is not a valid Rust identifier | – | yes |
| 8 | `8c9edf8` | Emit TypeScript type-parameter defaults on the type declaration | – | yes |
| 9 | `cf157a5` | Lower inner-position nullable types to `JsNullable` when `null` is named | – | yes |
| 10 | `985601b` | Keep the null arm of nullable parameters where omission cannot express it | 9 (`Nullish` tag) | no |
| 11 | `2a64afd` | Emit a sync `<name>_promise` form beside every async binding | 2 (`--fallible` catch rule) | no |
| 12 | `7cec9e7` | Add declaration-level `--export` roots with transitive closure | 1 (options struct) | no |
| 13 | `bb24e3d` | Give type-level generics defaults; bind only mentioned params on constructors and statics | 8 | no |
| 14 | `c73d7df` | Chase generic alias instantiations with their arguments substituted | – | yes |
| 15 | `0ecf144` | Carry construct-signature type parameters and the merged interface's body scope | 13 | no |
| 16 | `d2171b5` | Promote intersection aliases to interfaces; carry alias type parameters through promotion | – | yes |
| 17 | `942def1` | Keep the `Array<T>` form for `T[]` over a type parameter | 13 (generic element detection) | no |
| 18 | `30134e9` | Dynamic unions: literal payload variants and string-enum splicing | 4 | no |
| 19 | `2144dc7` | Dictionary helpers: float literals for numeric discriminants | – | yes |
| 20 | `0cd86b5` | Use each overload's own return type for its expansions | 11 (`_promise` form per expansion) | no |
| 21 | `f46fc60` | Promote constructor-shaped variables without an interface to classes | – (CONVENTIONS adjacency) | yes* |
| 22 | `678c014` | Merge a union-of-literals part of an intersection alias into the interface | 16 | no |
| 23 | `fd7acbc` | Collapse a global instance interface into its module-scoped constructor class | 21 | no |
| 24 | `6a56962` | Do not name factory variants after single-literal required fields | – (CONVENTIONS adjacency) | yes* |
| 25 | `881a3a2` | Resolve dotted namespace references; import the enclosing scope into namespace modules | 12 (`surface.rs`) | no |
| 26 | `a713fe3` | Add `@rust` JSDoc tags to force a Rust scalar for TS numbers | – (members.rs adjacency with 15) | yes* |
| 27 | `65ce36c` | Add `--annotate`: layer `@rust` and `@throws` tags from side `.d.ts` files | 26, 3 (README/AGENTS adjacency) | no |
| 28 | `e20536a` | Give the unsuffixed name to the first overload's shortest prefix-compatible expansion | – | yes |
| 29 | `ff68cdb` | Erase type parameters constrained to a primitive | 13, 15 | no |
| 30 | `5e8c264` | Declare inherited members on the child for parents beyond the first | 13 (`live_generic_args`), 16 | no |
| 31 | `5beabf1` | Synthesize `Named & { … }` parameter types as interfaces extending the named parts | 30 | no |
| 32 | `b455729` | Keep overload groups intact when merging var + interface declarations | 23 (merge code adjacency) | yes* |
| 33 | `2fc6fc8` | Lower enums inside generics, callbacks and arrays to their JS carrier | 4 | no |
| 34 | `da6fcff` | `--annotate`: a side declaration's type-parameter defaults replace the input's | 27 | no |
| 35 | `26d94f7` | Resolve `URIError` to `js_sys::UriError` and `DOMException` to `web_sys` by default | – (CONVENTIONS adjacency) | yes* |
| 36 | `53b0b7e` | Emit `declare const` statics in SCREAMING_SNAKE_CASE | – (snapshot adjacency only) | yes |
| 37 | `2cf01a1` | Give instance members their base name over statics and synthesized forms | 20 | no |
| 38 | `0e97679` | Lower awaited values like sync returns | 9, 11 | no |
| 39 | `b21ba1f` | Hoist inline method return literals; one union member per erased type | 31 (hoisting helpers) | no |
| 40 | `707f733` | Lower variadic callback types to the untyped `Function` | 13 (callback `Promise<T>` erasure sits in the same arm) | no |
| 41 | `b884448` | Rename types colliding with the Rust prelude, without a re-export | 8 (`generic-defaults` integration test) | no |
| 42 | `465667a` | Erase method type parameters no argument can infer | 20, 40, 11 | no |
| 43 | `8a85845` | Nested arrays: the inner array is the `Array` handle | 17 (same `Array` arm) | no |

`yes*`: the only conflict is a CONVENTIONS.md / fixture hunk adjacency; the code applies clean.
Posting out of order means re-resolving that hunk, nothing more.

## Per-PR notes

1. **`--error-type`.** Flag + `GenerateOptions::error_type`; `--errors-as-error` becomes a
   shorthand. Risk: low; additive. Review focus: `from_module_full(&GenerateOptions)` refactor.
2. **`--fallible`.** New policy flag, default unchanged. Getters annotated `@throws {X}` now emit
   `getter, catch` under *either* policy — the one default-output change in the commit, called
   out in the body. Risk: low-medium (signature layer).
3. **`--error-enum`.** New module `codegen/error_enum.rs` (339 lines), dispatch order of the
   `From<JsValue>` chain is the subtle part; the `Display` for thrown primitives (`String(value)`
   import) is folded in. Risk: medium; self-contained.
4. **String enum shape.** Fixes output wasm-bindgen rejects outright; big snapshot diff in
   workers-types (every enum). Compile-only integration test proves acceptance. Risk: low.
5. **ABV builder bound.** Two-hunk fix on top of 4's fixture. Risk: low.
6–7. **js_name fixes.** Runtime-correctness bugs (`_connect`, `WebSocketPair[0]`). Risk: low.
8. **Type-param defaults on `pub type`.** First of the generics series; integration test
   `generic_defaults`. Risk: low.
9. **`JsNullable` for named `null`.** Adds `ir::Nullish`; behavior change for `T | null` in
   inner position (`JsOption` → `JsNullable`). Risk: medium (IR change touches 8 files) but
   mechanical.
10. **`_with_null` arm.** Signature-expansion rule; grows the workers-types output by ~500 lines
    of `set_x_with_null`. Reviewers may ask whether setters need it; the body argues they do.
11. **`_promise` siblings.** +2750 lines in workers-types; doubles async surface. The design
    rejection of `IntoFuture` is in the body. Risk: medium, mostly size. Depends on 2 for the
    catch policy of the sibling.
12. **`--export` roots.** New `codegen/surface.rs`; third `--export` form. README example added
    in this audit. Risk: low-medium; closure rules are listed in the body.
13. **Type-level generics defaults everywhere.** The largest semantic commit of the generics
    series: handle-identity `PartialEq`/`Eq` for generic types, `extends` to bare head,
    constructors/statics bind only mentioned params, two wasm-bindgen workarounds. Risk: high
    for review attention; each sub-rule has a CONVENTIONS paragraph and fixture
    `generic-type-level`. Could be split (PartialEq/Eq vs. binding rules) if asked.
14. **Generic alias chasing.** Risk: low-medium; touches LUB and fan-out paths.
15. **Construct-signature type params.** Risk: low.
16. **Intersection promotion.** 203 lines in `populate.rs`; the review already tightened it to
    declared object parts only. Risk: medium.
17. **`Array<T>` for `T[]` over a type param.** Risk: low.
18. **Dynamic-union literal payloads.** Risk: low-medium; `codegen/mod.rs` enum builder.
19. **Float literals.** One-liner. Risk: none.
20. **Per-overload return types.** Reworks `build_signatures` to carry `overload_index`; the
    `_promise`/dedup scope fix from the audit is folded in. Risk: medium; the most central
    signature change on the branch. 37 and 42 build on it.
21. **Constructor-shaped vars.** Risk: low.
22–23. **Intersection union part / split-class collapse.** Parser post-passes. The reviewer's
    nit about signalling the split-class pattern via `class.extends = own name` still stands
    (noted in `parity/workers-types.md`); worth a sentence in the PR. Risk: low-medium.
24. **Factory naming.** Output-naming change (`new_bytes()` → `new()`); breaking for consumers
    of those names. Risk: low, but flag the rename.
25. **Dotted namespace refs.** Risk: low.
26. **`@rust` tags.** New IR (`RustScalar`), docs parsing. Design note on alias vs tag is in the
    body. Risk: medium; 412 lines, well-fixtured.
27. **`--annotate`.** New `parse/annotate.rs` (469 lines) with unit tests for every error path.
    Risk: medium. Reviewer question to expect: matching by bare name (recorded open problem).
28. **Primary overload naming.** Changes which expansion owns the bare name when the longer
    overload is declared first. Breaking rename for `R2Bucket.get`-style APIs; argued in body.
29. **Primitive-constrained type params erase.** `Binding::Erased` in the scope arena, −751
    lines in workers-types. Risk: medium (parse-side scope change).
30. **Inherited members on the child.** +2374 lines; duplicate extern fns are the accepted
    cost. Sibling-order rule folded in. Risk: medium; the classification-through-parents
    change deserves a careful read.
31. **`Named & {…}` params.** Builds on 30's `extends` with args. Risk: low.
32. **Overload groups in merge.** 41-line fix; CONVENTIONS line added in this audit. Risk: low.
33. **Enum carriers inside generics.** Risk: low.
34. **`--annotate` defaults.** Risk: low.
35. **`URIError`/`DOMException` defaults.** Risk: low; table change.
36. **SCREAMING_SNAKE statics.** Breaking rename for consumers (`caches` → `CACHES`). Risk: low.
37. **Instance names over statics.** Breaking rename (`json_2` → `json`, `json` → `json_static`).
    Risk: low-medium.
38. **Awaited values like sync returns.** Changes every async return type in the output
    (`Undefined` → `()`, `Boolean` → `bool`, …); removes the mono special case. The runtime
    verification claim is in the body; reviewers may want the integration test extended.
    Risk: medium-high by blast radius, low by mechanism.
39. **Return-literal hoisting.** Parser change; `never` → `Undefined`. Risk: medium.
40. **Variadic callbacks → `Function`.** Risk: low.
41. **Prelude collisions.** Risk: low; compile-only integration test.
42. **Uninferrable type params.** New `codegen/type_params.rs`; the rule and its three cases are
    in CONVENTIONS. Breaking for callers that wrote `run::<JsValue>()`. Risk: medium.
43. **Nested arrays.** Fixes a non-compiling lowering. Risk: low.

## Suggested posting order if not strictly sequential

The independent set (1, 4, 6, 7, 8, 9, 14, 16, 19, 28, 36, and the `yes*` ones after a
CONVENTIONS re-resolve) can go first as small PRs to establish review cadence. The three
chains that must stay ordered: errors (1 → 2 → 3 → 11 → 20 → 37/38/42), generics
(8 → 13 → 15 → 17 → 29 → 30 → 31 → 39), annotations (26 → 27 → 34).

## Wave 1 branches

Local only (not pushed), each cut from `origin/main` = `e139daa` with the single audit commit
cherry-picked and snapshots re-blessed against main; `just test`, `just clippy`, fmt check and
the wasm32 integration check pass on every commit; author `Guy Bedford <guybedford@gmail.com>`.
Where the audit commit's fixture or CONVENTIONS hunk assumed a branch-only feature, the branch
carries a main-compatible version (noted). PR body = the commit body; the paste-ready summary
below is the shorter form.

| Slug | Base | SHA | Audit # |
|---|---|---|---|
| `gbedford/string-enum-shape` | main | `df5e97e` | 4 |
| `gbedford/abv-builder-bound` | `gbedford/string-enum-shape` | `0dde763` | 5 |
| `gbedford/renamed-function-exports` | main | `1f64400` | 6 |
| `gbedford/accessor-js-name` | main | `0c731b1` | 7 |
| `gbedford/float-discriminants` | main | `3fe1653` | 19 |
| `gbedford/overload-primary-name` | main | `bc2ee7f` | 28 |
| `gbedford/builtin-error-defaults` | main | `a977e50` | 35 |
| `gbedford/const-static-names` | main | `e86fcc4` | 36 |
| `gbedford/nested-arrays` | main | `7e498ed` | 43 |

**`gbedford/string-enum-shape`** — String enums: the `Variant = "value"` shape, passed by value

> String-literal unions emitted `#[wasm_bindgen(js_name = "x")] X` variants, which wasm-bindgen
> rejects (it wants `X = "x"`), and enum arguments were borrowed although a `Copy` enum only
> crosses by value, so every member taking one failed to compile. Both fixed; `enum-args`
> fixture plus a compile-only wasm32 integration test. workers-types: every string enum takes
> the new shape and enum arguments drop their `&`.

**`gbedford/abv-builder-bound`** (stacked on the above) — Builder methods for ArrayBufferView
fields carry the setter's TypedArray bound

> An optional `ArrayBufferView` dictionary field gets a `set_x<T: TypedArray>(&T)` setter, but
> the fluent builder method forwarding to it was rendered with a concrete `&Object`, so the
> builder never type-checked. The builder method now carries the same bound; covered by the
> `enum-args` integration test.

**`gbedford/renamed-function-exports`** — Bind renamed function exports to their public name

> `function _connect(); export { _connect as connect }` published the Rust fn as `connect` but
> kept `js_name = "_connect"`, which the module does not export, so the import failed at
> instantiation. The rename is applied to the declaration and the alias stub suppressed.
> workers-types: `cloudflare:sockets`' `connect` binds correctly.

**`gbedford/accessor-js-name`** — Emit `js_name` for accessors whose key is not a valid Rust
identifier

> `js_name` was emitted only when the snake-cased name differed textually from the key, but
> sanitisation happens later in `make_ident`: `0` became `_0` with no `js_name`, so
> `WebSocketPair[0]` read property `_0` (undefined). Getter and setter now compare against the
> emitted ident. workers-types: `$public` and `@cf/...` accessors gain `js_name`.

**`gbedford/float-discriminants`** — Dictionary helpers: float literals for numeric discriminants

> A numeric-literal discriminant was baked into the factory through `f64::to_string`, which
> renders `0.0` as `0`, an integer literal Rust will not pass to an `f64` setter. Emits float
> literals. workers-types: `edgeRequestKeepAliveStatus` factories set `1.0` … `5.0`.

**`gbedford/overload-primary-name`** — Give the unsuffixed name to the first overload's
shortest prefix-compatible expansion

> With a required-options overload declared before the optional one (`R2Bucket.get`,
> `D1PreparedStatement.first`/`raw`) the longer form took the bare name and the short form
> fell through to numeric dedup (`get_2`, `first_2`). The primary is now the first overload's
> shortest expansion, or a shorter expansion whose leading parameter names match it; pure
> shortest-arity was rejected since it would hand `put(entries)` the name over `put(key, value)`.
> Renames the affected bindings. (On main the overload index is tracked locally in
> `expand_signatures`; the audit commit took it from a later change.)

**`gbedford/builtin-error-defaults`** — Resolve `URIError` to `js_sys::UriError` and
`DOMException` to `web_sys` by default

> `@throws {URIError}` was "unresolved" because the TS spelling is not in the `js_sys::*` glob,
> and `DOMException` needed a declaration or `--external` though every other web platform class
> defaults to `web_sys`. Both go through the external map's default tables (`JS_SYS_RENAMES`,
> `WEB_SYS_DEFAULTS`); the builtin scope iterates those instead of a duplicate list. (Fixture
> runs without flags on main: `try_decode` returns `Result<_, URIError>`.) No workers-types diff.

**`gbedford/const-static-names`** — Emit `declare const` statics in SCREAMING_SNAKE_CASE

> `declare const caches` became `pub static caches`, tripping `non_upper_case_globals` in every
> consumer, and wasm-bindgen drops attributes on foreign statics so the item cannot carry its
> own `#[allow]`. Statics follow the Rust convention with `js_name` binding the JS spelling.
> Breaking rename: workers-types' `console`/`crypto`/`caches`/`self` become
> `CONSOLE`/`CRYPTO`/`CACHES`/`SELF`.

**`gbedford/nested-arrays`** — Nested arrays: the inner array is the `Array` handle

> wasm-bindgen has no `Vec<Vec<_>>` ABI, so `string[][]` lowered to a `Vec<Vec<String>>` that
> fails to compile. The inner array now takes its inner-position form: `&[Array<JsString>]` /
> `Vec<Array<JsString>>`. New `nested-arrays` fixture (the audit commit extended the
> branch-only `generic-slice` fixture). workers-types: the Ai embedding outputs' `data:
> number[][]` read `Vec<Array<Number>>`; `basic`'s `HeadersInit` slice likewise.

Notes: `nested-arrays` applied on main without #7 (`accessor-js-name`), so it is not stacked.
`abv-builder-bound` is the one stack. `builtin-error-defaults` is the only branch with no
workers-types snapshot change.

## Wave 2 branch

`gbedford/error-model` on main, three commits in stack order, checks green on each, not pushed:

| SHA | Title |
|---|---|
| `8ee7b0e` | Add `--error-type <path>` for a configurable default error type |
| `0da9ffa` | Add `--fallible` policy: one Result binding per sync callable |
| `8c36c1b` | Add `--error-enum`: `@throws` types become one generated enum |

Bodies are the commit messages (#1–3 in the table above). Post as one PR with three commits or
three stacked PRs; the audit notes under 1–3 apply.

## Wave branches (3–10)

**The stack is linear.** The grouping as specified could not be kept as independent bases:
`generics` #17/#29 collide with `nullable-async`'s `Nullish` tag in `typemap.rs`, and wave 6's
#20/#37 need `nullable-async`'s `_promise` form. So the order is
`error-model → nullable-async → generics → declaration-shapes → inheritance-namespaces-unions →
annotations → inference-naming → inline-wrappers`, each branch based on the previous one. Two
commits moved from their assigned wave to keep audit order and avoid rewriting them: #29
(primitive-constrained type params) from `generics` to wave 6 after #25 (it edits `populate.rs`
/ `typemap.rs` lines that #16, #18 and #25 write), and #32 (overload groups in merge) from
`declaration-shapes` to wave 6 after #30 (its `parse/mod.rs` context is #30's). Four manual
conflict resolutions were needed (#24 and #25 CONVENTIONS adjacency, #29 and #39 vs #26's
`apply_rust_tags` lines in `members.rs`, #26 vs the same in reverse); correctness check: the
wave-8 tip plus the remaining wave-1 commits reproduces `agent/pr-audit`'s tree **byte-for-byte**,
and the wave-9 tip likewise reproduces `agent/errors`' source, docs and fixtures.

**Wave-1 dependencies.** Two posted wave-1 commits are needed underneath and are carried as the
first commit of the wave that needs them (they drop out once the wave-1 PR merges):
- `gbedford/string-enum-shape` (#39) at the bottom of `inheritance-namespaces-unions`: #33
  (enum carriers) refactors its `is_enum`, #18 (string-enum splicing) and #25 (namespaced enums
  pass by value) build on it.
- `gbedford/overload-primary-name` (#43) inside `inline-wrappers`, before the literal-keyed
  overloads commit, which edits its `primary_expansion` block. Carried as the audit-branch version
  (`e20536a`, which already assumes #20's `indices`), not the main-branch one.
Nothing else in waves 3–9 needs a wave-1 commit (verified by the byte-identical reproduction
above, where the other seven apply on top without touching a wave commit's files).

Every commit: `just test`, `just clippy`, fmt check, wasm32 integration check green; author
`guybedford@gmail.com`; not pushed.

| Wave | Slug | Base | Commits (audit #) |
|---|---|---|---|
| 3 | `gbedford/nullable-async` | `gbedford/error-model` | `15e0bd8` #9, `b86a21f` #10, `a8845f5` #11, `fb30a4c` #38 |
| 4 | `gbedford/generics` | `gbedford/nullable-async` | `3ee760c` #8, `d93a19f` #13, `f2fe5d4` #14, `d004b46` #15, `c960d0c` #17 |
| 5 | `gbedford/declaration-shapes` | `gbedford/generics` | `a57b2ee` #16, `9b38329` #21, `939a668` #22, `2928d6e` #23, `3569c6b` #24 |
| 6 | `gbedford/inheritance-namespaces-unions` | `gbedford/declaration-shapes` | `65ec244` (= wave-1 #4), `5feea72` #12, `03cca13` #18, `214d4fe` #20, `d33bbdb` #25, `c18bd36` #29, `4de5489` #30, `4e4f777` #31, `f39ad42` #32, `fcd4580` #33, `4201037` #37, `4bb19a1` #39, `f697e27` #40 |
| 7 | `gbedford/annotations` | `gbedford/inheritance-namespaces-unions` | `69a69af` #26, `66d9ac8` #27, `fa7c40d` #34 |
| 8 | `gbedford/inference-naming` | `gbedford/annotations` | `fe43227` #41, `7e35df4` #42 |
| 9 | `gbedford/inline-wrappers` | `gbedford/inference-naming` | `5974e45` (u64/i64 wrappers), `32ad680` (= wave-1 #28), `7ae9eb4` (literal-keyed overloads), `6c3d9d0` (`@throws {never}` on constructors) |
| 10 | `gbedford/builder-from-dict` | `gbedford/inline-wrappers` | `8659a3e` (builders implement `From<Dict>`; workers-rs-4's `462937b`) |

### PR bodies

**Wave 3 — `gbedford/nullable-async`: Nullable and async return lowering.**
Four changes to how absent values and promises reach Rust. `T | null` in an inner position
(`Promise<T | null>`, array elements, callback returns) lowers to `JsNullable<T>` instead of
`JsOption<T>`, which is empty only for `undefined`, so KV `get` and `getAlarm` no longer resolve
`Some(null)`; the IR records whether `null` was written. A required or non-trailing `T | null`
parameter keeps a `_with_null(&Null)` variant, since omission cannot express `new Response(null,
init)`. Every `Promise`-returning callable gains a sync `<name>_promise` sibling returning the
typed `Promise<T>`, for callers that need the call started synchronously (`waitUntil`,
`blockConcurrencyWhile` in a constructor); `IntoFuture` on the primary was rejected because it
fixes the error type to `JsValue`. Finally the awaited value uses the top-level return lowering
(`()`, `bool`, `String`, `Option<T>`, `Vec<T>`) since `JsFuture` only needs `FromWasmAbi`,
subsuming the per-mono special case. workers-types: `JsOption` → `JsNullable` on nullable
resolves, `set_x_with_null` setters, +2750 lines of `_promise` forms, and every async return
re-typed.

**Wave 4 — `gbedford/generics`: type-level generics usable by their bare names.**
Upstream declarations use type parameters as phantom refinements (`Request<CfHostMetadata, Cf>`,
`ReadableStream<R>`), and mandatory Rust generics made the bare names unusable. Every `pub type`
parameter now carries a Rust default (the lowered TS default, else `JsValue`), `extends` lowers to
the bare head, constructors and statics declare only the parameters their own signature mentions,
and generic types take handle-identity `PartialEq`/`Eq` since the derive's `T: Eq` bound on
`PhantomData<T>` fails for `JsValue` and reaches every subclass. Generic alias instantiations are
chased with their arguments substituted; construct-signature type parameters get their own body
scope (and a var-promoted class takes the interface's scope); `T[]` over a type parameter keeps
the `Array<T>` handle form, since there is no slice ABI for a generic element. Two wasm-bindgen
limits are worked around for async generics (`FromWasmAbi` bound; `Promise<T>` inside a callback
erases).

**Wave 5 — `gbedford/declaration-shapes`: declaration patterns upstream files use.**
`type X<P> = A & B & { … }` promotes to an interface extending the named parts (the whole
`IncomingRequestCfProperties` / `D1Result` / `DurableObjectStub` family erased before), carrying
the alias's type parameters; a parenthesised union-of-literals part merges with the union-literal
rules (`R2Objects.cursor`). Any variable with constructor shape becomes a class (`WebSocketPair`),
and a global instance interface collapses into its module-scoped constructor class
(`EmailMessage` + `cloudflare:email`). Factory names drop single-literal required fields, which
had produced `new_0_none_0________________false__false`; a discriminated union's discriminant
still names each branch. Breaking renames: `new_bytes()` → `new()` style.

**Wave 6 — `gbedford/inheritance-namespaces-unions`: output selection, overloads, inheritance,
unions.** Carries wave-1 `string-enum-shape` at its base. Declaration-level `--export <Name>`
roots with transitive closure make a large upstream file usable without emitting all of it.
Dynamic unions gain literal payload variants and splice string-enum members. Each overload
expansion keeps its own return type (and the `_promise` form, the dedup rendering and the body
scope follow the same overload) instead of the first overload's. Dotted namespace references
resolve and namespace modules `use super::*`. Type parameters constrained to a primitive erase
(`KVNamespace<Key extends string>` → `key: &str`). Members of parents beyond the first are
declared on the child since wasm-bindgen derefs to the first `extends =` only, dictionary
factories set every parent's fields, `Named & { … }` parameters synthesize an interface, and
var+interface merging keeps overload groups whole (`ReadableStream.getReader()` had vanished).
Enums inside generics/callbacks/arrays cross as their JS carrier. Instance members own their base
name over statics (`json` / `json_static`). Inline method return literals hoist into
`<Parent><Member>Return` dictionaries, with `never` lowering to `Undefined`. Variadic callbacks
lower to the untyped `Function`. Largest wave; the per-commit messages carry the per-commit
workers-types sentence.

**Wave 7 — `gbedford/annotations`: `@rust` tags and `--annotate` side files.**
`@rust <type>` on a member or `@rust <param>: <type>` forces a Rust scalar for a TS `number`
(`u16` status, `u32` limits), with wasm-bindgen converting at the boundary; `u64`/`i64` apply to
`bigint` only (lifted in wave 9). Upstream files carry no such tags and vendored edits do not
survive upgrades, so `--annotate <side.d.ts>` layers `@rust` and `@throws` onto the input by
declaration and member name; a side name the input lacks fails the run. A side declaration's
type-parameter defaults replace the input's, so the bare `Request` can read `cf()` as
`IncomingRequestCfProperties`. Design note on tag-vs-branded-alias is in the first commit.

**Wave 8 — `gbedford/inference-naming`: prelude collisions and uninferrable type parameters.**
A declared type named like the Rust prelude (`Result`, `Option`, `String`, `Vec`, …) takes the
`js_sys` collision rename (`Foo_` + `js_name`) with no `pub use` back, since that alias would
shadow the prelude just the same. A method-level `<T>` no lowered argument mentions is kept only
when the (awaited) return is exactly `T` / `T | undefined` / `T | null` — there the turbofish is
the cast — and otherwise erases to the default of the type it instantiates in the return, else
the method's own, else `any`, and leaves the signature (`run::<JsValue>()` → `run()`,
`setTimeout<Args>` drops `Args`). Breaking for callers that wrote the turbofish.

**Wave 9 — `gbedford/inline-wrappers`: the inline-wrapper policy (workers-rs-2's commits).**
Carries wave-1 `overload-primary-name` (audit form) before the literal-keyed commit. Two things
the extern ABI cannot carry get a thin generated wrapper over a private extern: `@rust u64`/`i64`
on a `number` (binding keeps `f64` under `<name>_js`, a `pub fn` converts with `as`, exact to
2^53, through `Option`/`Vec`/slices/`Result`/async; `_promise` and callbacks keep `f64`), and
overloads that differ only in a string-literal argument (`get(key, type: "json")` → `get_json`
baking the literal in; arms sharing a return type share one `&str` extern). Also `@throws
{never}` on a constructor now lowers to a non-`catch` constructor returning the bare type
(`Headers::new()`), under both policies. Applied clean on wave 8 apart from the #28 dependency.

**Wave 10 — `gbedford/builder-from-dict`: dictionary builders implement `From<Dict>`.**
`build()` consumes the builder and the builder has no getters, so a helper that needs to read
what was set so far (append a header, apply a preset) had to rebuild the object field by field.
Every generated builder now implements `From<Dict>`: the dictionary's getters and setters are
the read/modify step and `.into()` resumes the chain. Six lines of codegen plus a CONVENTIONS
section; the only snapshot change is the added `impl From<…> for …Builder` on every dictionary.
Unrelated to the wrappers, hence its own wave; it cherry-picks onto wave 9 clean, and its code
hunk applies on main alone (only branch-only snapshots conflict), so it could be cut as an
independent PR after a re-bless if wanted earlier.

## Session 9: final stack on the upstream bases, posted

**Base** (`base/final`, local, 29 commits on `origin/main` = `f050db9`): #39
`gbedford/string-enum-shape` rebased onto main (9 commits), then #47 (2), #42 (1), #43 (2),
#44 (2), #45 (4), #46 (4), then #51 `13b6319`, #52 `0b555cf`, #53 `f543c95`, #54 `9743bf8`, then
one integration commit. The PRs are posted on `gbedford/wasm-embed`, so each PR's diff shows the
wave-1 and #39 commits until those land; the first PR body says so.

Conflicts between upstream branches, and how they were resolved (all in `base/final`, nothing
pushed to those branches):

- **#39 vs main (#40/#41 merged)**: `emit_type_name` — #39 resolves qualified names from the use
  site, #40 from the ambient module's scopes; chained use-site → module scopes → file scopes.
  `parse/mod.rs` — #39 inserts namespace members into their namespace scope and the file, #40
  keeps ambient-module bodies private; a `home_scope` walk (nearest file or module-body scope)
  does both. #40's `type_arena` kind check (removed by #39) reads the registry instead.
- **#39 vs #42** (`cd933a3`): `synthesise_dynamic_union` gained `scope, from_module` in #39 and
  `dynamic_union_members` in #42; both applied. Test blocks appended at the same spot: kept both.
- **#45 vs main (#41)** / **#45 vs #46**: CONVENTIONS "Naming conversion" bullets merged;
  `sanitised_keys` integration test reads `VERSION` (upper-case static).
- **#46 vs #39**: the native-element array rule keeps #39's enum-repr substitution.
- **#51 vs #39**: `resolve_alias` moved to `GlobalContext` (#51) while #39 added `enum_kind` /
  `enum_repr` beside it; kept both. `enum_positions` and `nested_arrays` integration tests follow
  #51 (`Promise<Mode | null>` is `JsNullable`; `string[][] | null` argument is `&JsNullable<…>`).
  `array_elements_are_native_only_for_strings_and_numbers` (#46) expects `JsNullable` for
  `Id | null` under #51.
- **#45 vs #53** (integration commit `8ee1ca2`): the wat emitter spelled the non-ASCII static
  `déjà` into a `$id`, which WAT rejects — non-ASCII chars become `_u<hex>_`; and #45's fixture
  declared `value` in two modules, which #53's ABI (names carry no specifier) refuses — the
  fixture uses distinct names. The design clash itself (same JS name in two modules) is left to
  #53.
- Cargo: #51 adds a `web-sys` patch; the wave-1 branches moved the wasm-bindgen pin to
  `5c070ca` (wasm-bindgen#5376) — all five patches now point there.

**Waves** (34 commits, every commit `just test` / `just clippy` / fmt / wasm32 integration
check green on its own; `workers-types.rs` re-blessed per commit):

| Wave | Branch | Tip | Commits |
|---|---|---|---|
| 2 | `gbedford/error-model` | `d85f2b3` | `565ba66` `--error-type`, `f298bd8` `--fallible`, `d85f2b3` `--error-enum` |
| 3 | `gbedford/nullable-async` | `c48a7dc` | `a69fd3d` `_promise` siblings, `c48a7dc` awaited values like sync returns |
| 4 | `gbedford/generics` | `809a44e` | `9038e6e` #8, `841ab2a` #13, `90e93d7` #14, `142810c` #15, `809a44e` #17 |
| 5 | `gbedford/declaration-shapes` | `c43a789` | `9bf524a` #16, `e95ad9b` #21, `1761f62` #22, `2422f10` #23, `c43a789` #24 |
| 6 | `gbedford/inheritance-namespaces-unions` | `29fe72e` | `af6ce2a` #12, `d6d15bd` #20, `260383d` #25 (reduced), `f9edc64` #29, `54cd2ec` #30, `4c69ee8` #31, `632769d` #32, `2e23735` #37, `a6b136f` #39, `29fe72e` #40 |
| 7 | `gbedford/annotations` | `9530900` | `f3f1f6b` #26, `081ec47` #27, `9530900` #34 |
| 8 | `gbedford/inference-naming` | `85be297` | `f28a591` #41, `85be297` #42 |
| 9 | `gbedford/inline-wrappers` | `b9cde35` | `4ffe291` u64/i64 wrappers, `9c3a072` literal-keyed overloads, `b9cde35` `@throws {never}` on constructors |
| 10 | `gbedford/builder-from-dict` | `0ff7462` | `0ff7462` builders implement `From<Dict>` |

Dropped against session 8: **#9, #10** (replaced by #51's design; nothing re-expressed — the
`Nullish` reads were all `Nullable(inner, _)` patterns, and the two #51 follow-ups found in the
parked round are covered by #51's final version); **#18** (literal payload variants: #42's
`cd933a3` now produces the same `Bool(bool)` / `F64(f64)` shapes); **#33** (enum carriers in
generics/callbacks/arrays: #39's `8ef8329`); the carried wave-1 **#4** and **#28** (in the
base). **#25** is reduced to global namespaces (a `pub mod` path only when the namespace is
emitted as one; #39's flattened module namespaces keep the bare name, nested enums defer to
#39's carrier rule; #39's unit test now expects `codes::Region` for a global namespace).

Fixes folded into their commits during the rebase: #20 rotates `indices` with the primary
expansion (the main-form #43 rotated only `deduped`, so `R2Bucket.get(key)` reported overload
0's return type); #23 moves the absorbed interface's scope onto the class so references from the
global scope still resolve under #40's private module scopes; #12 resolves roots through
ambient-module scopes (`--export EmailMessage` for a class declared in `cloudflare:email`); #29
teaches #53's ABI resolver about `Binding::Erased`; #23 updates #53's
`real_constructor_wins_over_dictionary_new_across_roots` (the collapsed class is one root); #38
updates the `builtin_errors` / `nested_arrays` integration tests to awaited `String` / `Vec`.

**workers-rs** (`gbedford/tgw-workers-types` on `a2faa1b`): `a369845` regenerates from the stack
tip, adapts `ext/` and the port to `&JsNullable` arguments, and moves the `wasm-bindgen`
submodule to `1ea319f65` = reinit branch `870389dd3` + wasm-bindgen#5376 `5c070ca` (local
cherry-pick; needs pushing to the fork before this branch is shared). `chomp test:ts-gen-worker`
and `-http`: 194 passed / 4 skipped each. Migration deltas are in `parity/workers-types.md`.

### Posted

`gh stack` #70 on `gbedford/wasm-embed`: #61 error-model → #62 nullable-async → #63 generics →
#64 declaration-shapes → #65 inheritance-namespaces-unions → #66 annotations → #67
inference-naming → #68 inline-wrappers → #69 builder-from-dict. Titles/bodies set via the REST
PATCH (the per-wave prose above plus the base note and the drops), all undrafted.

## Restack (session 10)

Analysis only: nothing rebased, nothing pushed, branches untouched. `origin/main` = `a306021`.
The stack is 63 commits over `origin/gbedford/wasm-embed` (`9743bf8`, still on `f050db9`): 24
wave-1 copies, our copies of #51–#54 plus integration commit `8ee1ca2` (5), and our 34.

Method. Dry run: each commit cherry-picked in order with `git merge-tree` onto `origin/main` plus
our #51–#54 copies (the new #51–#54 tips are not out yet; this is the nearest stand-in). Each
conflict was recorded, then the chain continued with `-X theirs` so later commits keep the context
they were written against. Superseded check: main's `ts-gen` (`a306021`) and our tip run on every
fixture our commits add, on ad-hoc inputs, and on `workers-types.d.ts`.

### Base commits

- **Wave-1 copies (24): merged, drop.** For #42, #44, #46 and #47 our copy's code/doc diff is
  identical to the squash. Three merged in a different form:
  - **#43 `22d3c3e`** uses the main-form `first_overload_len` counter, not our `indices`. Our
    `d6d15bd` and `9c3a072` must reintroduce the per-overload index (see below).
  - **#45 `2191f49`** changed design: statics keep the JS casing with
    `#[allow(non_upper_case_globals)]` (wasm-bindgen#5377), so `console`/`caches`/`crypto`, not
    `CONSOLE`/`CACHES`.
  - **#39 `88d0863`** landed with Guy's resolution, not our local `home_scope` one.
- **wasm-bindgen pin.** Main pins `2df31ba` (#5376 as `2a0fd20e3` + #5377 + three commits:
  same-named enums/dynamic unions across modules, shared name space). Our base pins `5c070ca`,
  and workers-rs' `wasm-bindgen` submodule (`1ea319f65`) must move to the reinit branch + `2df31ba`.
- **#51–#54 copies on main** (for whoever rebases them):
  - `03f59c2` (#51) conflicts in `Cargo.toml`, `integration-tests/build.rs`, `package(-lock).json`,
    `codegen/classes.rs` (2 hunks), `codegen/typemap.rs` (4), `parse/literal_union.rs` (1).
  - `3061943` (#53) conflicts in `Cargo.lock` only.
  - `6d01332` (#52) and `fa0d514` (#54) apply clean.
  - `8ee1ca2` is still needed: main's `static-names` fixture still has `déjà` and `value` in two
    modules. It belongs on the #53 rebase.

### Classification of our 34

Superseded 3, superseded with a residual 2, drop recommended 1, overlapping 13, clean 15 (several
need a compile fixup).

"Conflict" lists code/doc files only. Snapshot conflicts are re-blessed and listed only where the
re-bless changes the commit's meaning.

| Wave | SHA | Commit | Class | Dry-run conflict | Re-expression / note |
|---|---|---|---|---|---|
| 2 | `565ba66` | `--error-type` | clean | `typemap.rs` (adjacency) | #83 added a 4th `from_module_full` test call site → `&GenerateOptions::new()` |
| 2 | `f298bd8` | `--fallible` | clean | `classes.rs` (2) | `used_names` → `names` (#86); tests need `rust_name: None` (#94) and #80's `prop()` helper needs `throws` |
| 2 | `d85f2b3` | `--error-enum` | overlapping (#83, #86, #101) | – (won't compile) | collect over `surfaces.values()` (#101 already drops unexported module statics/ctors), drop the Namespace recursion (members are `TypeId`s now), reserve `JsException` via `names.claim` instead of the local-type check |
| 3 | `a69fd3d` | `_promise` siblings | overlapping (#86) | – (won't compile) | `dedupe_name(…, used_names)` → `names.declare(…, Suffix::Snake)`; meaning unchanged |
| 3 | `c48a7dc` | awaited values like sync returns | clean | `typemap.rs` (test) | take our deletion of the mono test cases; ours is better on main's own enum cases (`Promise<Level>` → `Level`, main `JsString`) |
| 4 | `9038e6e` | type-param defaults | clean | `classes.rs` (adjacency with #81 `derive_eq`) | – |
| 4 | `841ab2a` | type-level generics defaults | overlapping (#81) | `classes.rs` (2) | our bare-head `extends` hunk is superseded by #81's `extends_list`; our handle-identity `PartialEq`/`Eq` covers what #81's `derive_eq` does, so remove `derive_eq` and change #81's CONVENTIONS line (**decision**). Main derives `Eq` on `AbortSignal extends EventTarget` with a generic `EventTarget` (from reading the code, not compiled) |
| 4 | `90e93d7` | generic alias chasing | overlapping (#89) | `mod.rs`, CONVENTIONS | fold #89's `Ns.Alias` branch into `resolve_alias_instance` via `gctx.resolve_path`; otherwise `ns::Alias<JsString>` against a param-dropped alias is E0107 |
| 4 | `142810c` | construct-signature type params | overlapping (#74/#95) | `members.rs` | #74 deleted `convert_construct_signature`: drop that hunk, keep the `extract_var_members` / class-ctor / `merge_class_pairs` hunks; re-bless may add ctors on `XConstructor`-merged classes |
| 4 | `809a44e` | `Array<T>` for `T[]` over a type param | **superseded (#96)** | `classes.rs`, `functions.rs`, `typemap.rs` | #96 decides `slice_to_array` from the lowered element, so `&[T]`/`Vec<T>` compile; main's form is better. Drop |
| 5 | `9bf524a` | intersection aliases → interfaces | **superseded by #81, residual** | CONVENTIONS, `classes.rs`, `populate.rs` | #81 hoists `A & B & {…}` with extends and alias params, and is better on `string & {__brand}` → `JsString`, `T & {…}` → `T`, unbound parents. Lost: an **inline conditional part** (`(T extends … ? … : unknown) & {…}`) makes #81's `inline_shape` give up, so **`Fetcher<T, R> = JsValue`** on main, `DurableObjectStub` extends `Object` only, `Loopback*` erase. Residual commit: skip conditional/`object`/`unknown` parts in `inline_shape`'s Intersection arm with a warning. Rewrite the message |
| 5 | `e95ad9b` | constructor-shaped vars → classes | overlapping (#74) | `populate.rs` | main still emits `static web_socket_pair: Object`; move into `push_static_side_class` (inline and `XConstructor` forms) for the no-instance-interface case; compare resolved decls, not names (#95 renames) |
| 5 | `1761f62` | union-of-literals part of an intersection | overlapping (#81) | – (clean only via our 9bf524a) | main: `R2Objects = JsValue`. Add a union part to `InlineShape::Intersection` (Join branches, then Meet). **Decision:** explicit `truncated: boolean` beside a joined `true \| false` becomes a bounded intersection under Meet; ours lets the explicit member win |
| 5 | `2422f10` | collapse global interface into module ctor class | **drop recommended** | – (same region as e95ad9b) | its reason (two flat `pub type EmailMessage`) is gone with #94; a collapse contradicts main's "global and module-scoped resolve independently". Re-expressed e95ad9b gives `cloudflare::email::EmailMessage extends = super::super::EmailMessage` (**decision**) |
| 5 | `c43a789` | factory names drop single-literal fields | clean | – | main still emits `new_0_none_0______________()`, `new_bytes()` |
| 6 | `af6ce2a` | `--export <Name>` roots | overlapping (#94, #101) | CONVENTIONS, `mod.rs`, `main.rs` | main reads `--export Bucket` as a module specifier. Closure feeds `exported_modules` before `from_module_full`, gate `collect_surfaces` by `should_emit`; module decls the closure reaches go to their nested `pub mod`, not lifted (CONVENTIONS/README "lifted as if listed" changes); fold in 260383d's `surface.rs` hunk. **Bug in ours:** `--export TailStream.TailEvent` (the documented example) fails, since namespace members aren't roots |
| 6 | `d6d15bd` | each overload's own return type | overlapping (#43 main form, #86) | `signatures.rs` (2) | keep an `indices` vec parallel to `deduped` beside `first_overload_len`, rotate with the primary; keep #86's `names.declare`. Main still has R2Bucket `get(key)` → `JsOption<R2Object>` (ours `Option<R2ObjectBody>`) and leaks `use JsValue as U;` |
| 6 | `260383d` | global namespace refs, `use super::*` | **superseded (#88, #83, #89, #94)** | CONVENTIONS, `mod.rs` | main matches ours on the fixture and is better on namespace aliases, nested `js_namespace`, namespaces in modules. Drop (its `surface.rs` hunk moves to af6ce2a) |
| 6 | `f9edc64` | erase primitive-constrained type params | clean + fixup | – (won't compile) | #81's `is_extendable` needs a `Binding::Erased` arm. **Bug in ours:** `interface W<K extends string> extends Base<K>` emits `new(b: &K)` with `K` unresolved; substitute the erased arg through `extends` args |
| 6 | `54cd2ec` | inherited members beyond the first parent | overlapping (#81) | `classes.rs`, `converters.rs`, `parse/mod.rs` | collect next to #81's `extends_list`; add `type_arguments` at oxc 0.154's `convert_ts_type_name_to_ref`. **Regression in ours:** a parent's required readonly field empties the child's factory (`QueryResult::new(results)` → `new()`, 10 `StreamError` subtypes lose `name`); main is right. Without 841ab2a's `derive_eq` removal, the re-bless also drops `Eq` on `WebSocket`/`RequestInitCfProperties` |
| 6 | `4c69ee8` | `Named & {…}` params → interfaces | **superseded (#81)** | `members.rs` | identical names and shapes on main (`R2BucketOptions extends R2GetOptions`). Drop |
| 6 | `632769d` | keep overload groups in var+interface merge | clean | – | main still drops `ReadableStream.getReader()` (keeps only the options form) and `Response.text()` in `basic` |
| 6 | `2e23735` | instance members own the base name | overlapping (#86) | `classes.rs` (2) | pre-`claim` instance names in `generate_extern_block`'s `LocalScope`, `release` on emit; main gives `json` to the static and `json_2` to the instance |
| 6 | `a6b136f` | hoist inline return literals | overlapping (#81) | CONVENTIONS, `members.rs` | route through `try_synthesize_inline_param` (`<Member>Return`), port the enclosing-type-param capture (also fixes main's non-generic `CursorOpts` for `{value: T}` params). Main reuses one union enum across sites (`ImageOptions.border -> Option<NextReturnKind>`). **Decision:** hoist `Named & {…}` returns, or bound them to `Named` as #81 does |
| 6 | `29fe72e` | variadic callbacks → `Function` | clean | – | main: `set_timeout(&Function<fn(Array) -> Undefined>)` |
| 7 | `f3f1f6b` | `@rust` tags | clean + 2 fixups | – | call `apply_rust_tags` in `merge::extract_var_members` (#74 path, else tags on `FooConstructor` are dropped); add `RustScalar` to `is_native_array_element` (#96, else `&[u8]` loses `slice_to_array`) |
| 7 | `081ec47` | `--annotate` | clean + fixup | – (won't compile) | #81 already added `GlobalContext::get_type_mut`: drop ours. After #74 side files annotate `declare abstract class Foo { static … }`, not `FooConstructor` |
| 7 | `9530900` | `--annotate` defaults | clean | – | – |
| 8 | `f28a591` | Rust prelude collisions | **superseded mechanism (#86/#89), residual** | CONVENTIONS, `classes.rs`, `typemap.rs` | main has `RUST_PRELUDE_RESERVED` (`Option`, `Result`, `String`, `Vec`) through `LocalScope`, also for aliases. Residual: extend the list with what our stack spells bare (`From` for 0ff7462, `PartialEq`/`Eq`/`AsRef` for 841ab2a) |
| 8 | `85be297` | erase uninferrable method type params | clean | `functions.rs` (trivial) | keep our erase block, main's `LocalScope` |
| 9 | `4ffe291` | u64/i64 inline wrappers | clean + fixups | – | claim the private `<name>_js` via `LocalScope` (today `@rust u64 size` + `sizeJs` emit two `size_js`); `WideScalar` in `is_native_array_element` |
| 9 | `9c3a072` | literal-keyed overload wrappers | overlapping (#43 main form, #86) | `signatures.rs` (4) | `arms` parallel to `deduped` beside `first_overload_len`; needs d6d15bd's per-overload index; `dedupe_name` → `names.declare`. **Bug in ours:** under mono the `_js_string` sibling re-emits `get_text`/`get_json` |
| 9 | `b9cde35` | `@throws {never}` on constructors | clean | – | main's CONVENTIONS still says "ignored on a constructor" |
| 10 | `0ff7462` | builders `From<Dict>` | clean | – | needs `From` reserved (f28a591 residual) |

Dry-run straight onto main without the #51–#54 copies adds conflicts only where those copies sit
underneath (`565ba66` `mod.rs`/`main.rs`, `d85f2b3` AGENTS, `2422f10` `abi/mod.rs`, `f3f1f6b`
`ir.rs` from #51's IR, more snapshots), so the table above is the representative one.

Restacked order (drops removed, residuals in place):
- error-model: `565ba66` `f298bd8` `d85f2b3`
- nullable-async: `a69fd3d` `c48a7dc`
- generics: `9038e6e` `841ab2a` `90e93d7` `142810c`
- declaration-shapes: conditional parts (9bf524a residual) `e95ad9b` `1761f62` `c43a789`
- wave 6: `af6ce2a` `d6d15bd` `f9edc64` `54cd2ec` `632769d` `2e23735` `a6b136f` `29fe72e`
- annotations: `f3f1f6b` `081ec47` `9530900`
- inference-naming: prelude list (f28a591 residual) `85be297`
- inline-wrappers: `4ffe291` `9c3a072` `b9cde35`
- builder-from-dict: `0ff7462`

### Regressions: main vs ours on the same input, and what workers-rs relies on

Each line: what main produces, what ours produces, and what the workers-rs port (`ts-gen-worker/src`
`ext/` and `test/src`) uses today.

- **`Fetcher`.** Main: `pub type Fetcher<T, R> = JsValue`. Ours: an interface with
  `fetch`/`connect`. The port relies on ours: `env.rs` `remote()`/`assets() -> Fetcher`, and
  `DurableObjectStub` reaches `fetch` through `extends = Fetcher`. Needs the 9bf524a residual.
- **`WebSocketPair`.** Main: `static web_socket_pair: Object`. Ours: a class. The port relies on
  ours: `WebSocketPair::new()` in `ws.rs`, `counter.rs` ×2, `container.rs`, `ext.rs`. Needs
  e95ad9b.
- **`ReadableStream.getReader()`.** Main: missing. Ours: present. The port relies on ours:
  `ext/streams.rs`, `request.rs`, `socket.rs` ×3. Needs 632769d.
- **`setTimeout` callback.** Main: `&Function<fn(Array) -> Undefined>`. Ours: `&Function`. The
  port relies on ours: `ext/delay.rs` and `test/util.rs` pass a plain `js_sys::Function`. Needs
  29fe72e.
- **`KVNamespace`.** Main: `KVNamespace<Key>` with no default. Ours: bare. The port relies on
  ours: `KVNamespace` is spelled bare throughout `env.rs`, `kv.rs` and `ext/kv.rs`. Needs f9edc64
  plus 9038e6e/841ab2a.
- **`IncomingRequestCfProperties` (and `D1Result::new`).** Main: no members from parents beyond
  the first. Ours: accessors such as `country()`. The port relies on ours (`request.rs`,
  `ext/interop.rs`, `ext/response.rs`). Needs 54cd2ec with its factory regression fixed.
- **`R2Objects`.** Main: `JsValue`. Ours: an interface. The port relies on ours: `test/src/r2.rs`
  `list()` reads `objects()`/`truncated()`/`cursor()`. Needs the 9bf524a residual + 1761f62.
- **Statics (#45 as merged).** Main: `console`/`caches`/`crypto`/`performance`. The port uses
  `CONSOLE`/`CACHES` (`ext/console.rs`, `lib.rs`, ~15 test sites). Port change after the restack;
  migration-table delta.
- **Declared modules (#94).** `connect`, `Socket` and `EmailMessage` move to
  `cloudflare::sockets::…` / `cloudflare::email::…`. Today they are flat: `test/src/socket.rs`
  calls `connect(…)` and `#[event(connect)]` takes `Socket`, through `ts_gen_worker::*`. Needs
  `pub use` re-exports in `ts-gen-worker/src/lib.rs` or path changes; migration-table delta.
- **`D1Result.results()`.** Main: `Vec<T>` (809a44e dropped). Ours: `Array<T>`. The port's
  `.results().into_iter()` works with both.

Main bugs our stack fixes (worth raising upstream regardless):
- one dynamic-union enum reused across sites (a6b136f);
- `use JsValue as U;` leak with unresolved overload type params (d6d15bd);
- possibly `Eq` derived over a generic parent (841ab2a; from reading, not compiled);
- `@throws {never}` constructors still `catch` (b9cde35).

Both main and ours collapse constructor overloads from `var X: { new(a); new(a, b) }` into one
(`extract_var_members` returns a single ctor).

### Decisions for Guy

1. **841ab2a:** replace #81's `derive_eq` (no `Eq` when the first parent has TS args) with the
   handle-identity `PartialEq`/`Eq` impls, editing #81's CONVENTIONS line.
2. **2422f10:** drop, and let `cloudflare::email::EmailMessage` extend the global interface
   (recommended), or re-express the collapse under the export map.
3. **1761f62:** explicit member vs joined literal union under Meet: let the explicit member win
   (ours), or teach `simplify_intersection` that `boolean ⊇ true | false`.
4. **a6b136f:** hoist `Named & {…}` in return position (ours today) or bound it to `Named` as #81
   does for parameters of named parts only.
5. **f28a591 residual:** the full prelude list, or only the trait names the generated code spells
   bare (`From`, `PartialEq`, `Eq`, `AsRef`).

## Restack 4 (session 16): the open PRs on main `48098a9`

Merged since the restack-3 base `2796dd3`: #51, #61, #62, #63, #64, #65, #155, #157, #176, #181
(the literal-keyed half of #68, squashed from `a06347a`/`4e1bc76`/`fb6dd04`/`e6bc424`/`3063f43`),
#122 and #110 from the mono work, and #170/#177/#179 from main. Open, each rebased onto its
declared base in `restack/<branch>` (local, not pushed):

| PR | branch | base | own commits | note |
|---|---|---|---|---|
| #52 | `gbedford/exports` | main | 2 | |
| #53 | `gbedford/wasm-abi-model` | #52 | 7 | `d327f74` "Describe ABI overload names" carried from the mono-prototype copy |
| #54 | `gbedford/wasm-embed` | #53 | 6 | |
| #66 | `gbedford/annotations` | main | 8 | |
| #67 | `gbedford/inference-naming` | main | 6 | merged as `48098a9` in the second pass; dropped from the stack |
| #68 | `gbedford/inline-wrappers` | #66 | 4 | the literal-keyed commits dropped as landed (#181); duplicate `## Inline wrappers` heading folded into the u64 commit |
| #69 | `gbedford/builder-from-dict` | #68 | 1 | |

The wasm ABI PRs stay underneath because the mono commits edit `src/abi/mod.rs`. #180
(`--promise-only`) is left out of the stack (Guy).

**Linear stack** (`gbedford/mono-prototype`): main → [option-null-reads layer, see below] → #52 →
#53 → #54 → #66 → #67 → #68 → #69 → the mono segment. Of the 48 mono commits, dropped as already
on main or superseded: `6e22f0c` (#122), `8b6305e`, `5fdbf2b`, `6d1dd4f`, `8a53f8c` (empty),
`8c4e184` (pin bump), `17df4e9` and `64e785e` (main's `null_arms.rs` design), `448388f` (main
already gives a written `null` its own `_with_null` arm everywhere), `2b9d924` (main's factories
already take a branch's required field bare). Partial drops inside kept picks: Partial synthesis
(`76b070a`, main's #177 evaluates `Partial<..>` itself, naming the result
`PartialKvNamespaceGetOptionsUndefined`), the trailing-null retain (`1549ee7`), `head`
(`e4d3fde`), Partial getter `Option` (`7ec1b4c`), the inline `abi_bounds` pass (`1fb83ab`,
`9464827`; main's `state_abi_bounds` states them). `010f6f6` (per-mono reads fold both empties
into `Option`) was first dropped as contradicting main's exact `Nullable` reads, then re-expressed
as the option-null-reads layer once Guy settled the read rule (below).

New on the stack in this session:
- "Per-mono, type parameters in Function signature slots are MonoGeneric": js-sys's
  `Function<fn(..)>` slots are bounded by `MonoGeneric` since wasm-bindgen#5390, so a bare type
  parameter in a slot (`fn(Request<Cf>, Env, ExecutionContext<Props>)`) gets `MonoGeneric` and a
  nested one `'static`, on imports and helper impls. Without it the regenerated bindings had 608
  errors.
- The option-null-reads layer (a ts-gen main PR candidate, both modes): reads of `T | null`
  (returns, awaited values, getters, callback arguments) are `Option<T>`, `T | null | undefined`
  stays `Option<Nullable<T>>`, `any | null` reads `Option<JsValue>`; writes and type arguments keep
  `Nullable`/`_with_null`. Pairs with wasm-bindgen#5428, which makes `Option<T>` read `null` as
  `None` again. Reason: `IntoWasmAbi` must choose a representation (`None` → `undefined`), so the
  write side needs `Nullable`; `FromWasmAbi` chooses nothing, and a `T | null` read loses no
  information as `Option<T>`. On the workers-types surface this folds 81 of 97 `Nullable`s; the 16
  left are 5 `?: T | null` dictionary getters (`ResponseInit::web_socket`, `RequestInit::fetcher`
  / `signal`, `KVNamespaceListOptions::prefix` / `cursor`), the 10 KV bulk `Map<String,
  Nullable<_>>` type arguments and the `SqlStorageValue` alias.

Regressions of the regenerated bindings against the prototype that the restack does not fix:
`number | bigint` lowers as `f64` plus a `_with_big_int` arm (15) instead of
`impl JsBigIntOrNumberLike`; the typed KV read wrappers take `&KVNamespaceGetOptions<String>`,
whose builder asks for the `type` the wrapper then overwrites; `Exclude<CertVerificationStatus,
"NONE">` is named `ExcludeCertVerificationStatusNone` (#177's literal naming).

Second pass of the session, after ts-gen main moved to `48098a9` (13 commits: #67 merged, #182–#193):
the read rule became a standalone PR branch `gbedford/option-null-reads` (2 commits on main:
"Reads are Option<T>; --strict-nullable keeps the exact carriers", "Bound a generic Option<T> read
whole") and the stack was rebuilt as main `48098a9` → that branch → #52 → #53 → #54 → #66 → #68 →
#69 → 46 mono commits (76 over main; `restack/inference-naming` dropped as merged, old tips tagged
`old/<name>`). Guy's rule, final: every read of an empty-admitting type is `Option<T>` — `T | null`,
`T | undefined` and `T | null | undefined` alike (audited on workers-types: every both-empties read
is an optional nullable dictionary member where both mean "no value"); writes and type arguments
keep `Nullable<T>` / `_with_null`; `--strict-nullable` restores the exact read carriers (the four
nullability fixtures carry a `strict.rs` snapshot, byte-identical to the old `default.rs`). A
top-type read (`any | null`, `x?: any`) stays bare `JsValue` in both modes (#188's rule applied to
every empty), so the per-mono `cf?: any → Option<JsValue>` sub-part of "Iterable arguments as
slices, native alias leaves, optional any as Option<JsValue>" was dropped as contradicting the
layer; other erased reads take `Option<JsValue>`. The integration-tests pin moves to wasm-bindgen
`gbedford/mono-prototype` `3a62aa7d4` (#5428 is the wasm-bindgen half of the rule).

Folded into the PR layers on the rebase: #52 lowers a provided function's *return* with a new
`TypePosition::EXPORT_RETURN` (owned like a return, exact like a write), so `export function
find(): string | null` is `-> Nullable<String>` instead of `Option<String>` handing JS `undefined`
(fixture, app and node check extended; the earlier "provides app takes Option" adaptation folded
in too); #66 and #68 absorb the `Option<u32>` / `Option<u64>` and `Box_` test adaptations. Mono
picks trimmed against main: "overload suffixes imply the primary arm and name dictionary arms by
parameter" keeps only the per-mono dictionary-by-parameter half (#191 covers implied primary arms);
"a field's literal choice within one shape is a factory argument" replaces #190's
`new_next`/`builder_next` under mono only (flagged); "nullable union returns are Option of their
enum" kept, since main's equivalent lives only in getters. None became empty.

Verification: PR-layer tip (#69) `just test-integration` 149/0 plain and 150/0 under
`js_sys_unstable_apis`, node provides check ok on both; mono tip `dc33537` fmt/clippy/`just test`
(402 + 1 + 28 + 3) green, `just test-integration` under the cfg 40 suites / 150 passed / 0 failed.
`tests/snapshots/workers-types`: `Nullable<` mono 85 → 12, default 104 → 49; `Option<JsValue>` mono
34 → 6, default 9 → 8.

The hand-written integration tests at the mono tip (`integration-tests/tests/*_mono.rs`,
`promise_builtins.rs`, …) had never compiled against a wasm-bindgen carrying the whole prototype;
"Rebase onto main: integration tests follow the restacked output" brings 15 files to the current
output (`Promise<T, E>` with no `_promise` twins, `Option<T>` reads, borrowed `&T` generic
arguments, `new_typed` constructors). With it `just test-integration` passes at the tip (64 test
binaries, 91 runtime tests, the `provides_namespace.mjs` check). Found on the way, not fixed here:
a `#[wasm_bindgen]` enum has no `&Enum` ABI (the macro emits `IntoWasmAbi` by value only, where
primitives also get `impl IntoWasmAbi for &T`), so `Holder::<Mode>::new(&Mode::Slow)` and every
other write of an enum type argument through `T: RefIntoWasmAbi` is uncallable — a wasm-bindgen
change, tracked in `mono-prototype-patches.md`; CONVENTIONS' `@rust` and "Inner positions"
passages still describe `Promise<Number>` / `JsNullable<T>` inner carriers where the blessed mono
snapshots emit `Promise<u32>`, `Vec<Option<u16>>`, `Object<Option<f64>>` (doc drift to fix when
the layer PRs are cut); `js_sys::Object::values` returns `Result<Array<T>, _>` only under
`js_sys_unstable_apis`, so `.iter()` on it silently iterates the `Result`.

