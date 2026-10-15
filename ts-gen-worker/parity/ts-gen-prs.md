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

## Wave branches (3–9)

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
