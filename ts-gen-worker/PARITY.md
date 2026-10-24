# ts-gen-worker parity tracking

Goal: `ts-gen-worker` reaches parity with the `worker` crate's existing surface
so that a `worker` user migrates with minor changes, and the `test/` suite runs
against it. New APIs are out of scope; only what `worker` exposes today.

Principles:

- The generated bindings are the API. Ergonomics are extra `impl` blocks on the
  generated types, never wrapper types mirroring them.
- Gaps are closed by ts-gen features (one PR each; the `ts-gen` submodule pins
  `gbedford/mono-prototype`, the open PRs restacked on main with the mono
  prototype commits on top) or by hand-written impls in this crate. Each
  choice is justified where it lands.
- The branch is public as the draft PR workers-rs#1103; the tracking notes stay
  on the history branch and are not part of the PR.

Per-topic notes live in `parity/*.md`; agents own one file each and this file
is the merged index.

## Status

| Area | State | Notes |
| --- | --- | --- |
| Error model (`--error-type`, `--fallible`, `@throws` enum) | integrated; plus `JsNullable` results, `_with_null` forms, `_promise` companions | `parity/errors.md` |
| Input from `@cloudflare/workers-types` | integrated: bindings generated from `node_modules/@cloudflare/workers-types/index.d.ts` + `types/annotations.d.ts` (`--annotate`: `@throws`, `@rust` integer hints, type-parameter defaults); `types/worker.d.ts` deleted; 19.8k generated lines | `parity/workers-types.md` |
| `Error`, `#[event]` macros, ergonomic impls | integrated; `#[event(connect)]`, `Response::into_parts`, builder `header/cors/body/from_websocket`, `Request::new_with_method`, serde storage helpers; `MIGRATING.md` written; 192 tests green on both modes | `parity/crate.md`, `MIGRATING.md` |
| ts-gen PRs | merged: #39–#47, #51, #61–#65, #155, #157, #181 (the literal-keyed half of #68), #122, #110; open: #52–#54 (wasm ABI), #66–#69, rebased onto main `b0af305` in session 16, under a new option-null-reads layer (reads of `T \| null` are `Option<T>`) | `parity/ts-gen-prs.md` "Restack 4" |
| Test suite port | integrated: all 26 spec files run (187 tests; 4 skipped are upstream's own gates); `signal`, `container`, `panic` ported, trap recovery via the wasm-bindgen fork's reinit | `parity/test-port.md`, `parity/errors.md` Session 4 |

## Open problems for Guy

Items agents could not resolve without a decision. Each has a proposal. Details in the linked
file.

- **`JsException` dispatch is `instanceof`-only.** Cloudflare binding errors (`ImagesError`,
  `ArtifactsError`, …) are `interface X extends Error { readonly name: "X" }` with no constructor,
  so when they enter the `--export` roots the enum needs a `name`-discriminated arm. Out of
  scope today (no `worker` surface uses them); ts-gen change when it comes.
- **How authoritative must `@throws` be?** Only Fetch-spec-pinned classes are annotated so far.
  Runtime-specific throws (KV/R2/D1, `DOMException` names) need a policy; a wrong annotation lands
  the value in `JsException::Other`, never a wrong variant. (`parity/errors.md`)
- **The branch depends on unmerged wasm-bindgen PRs**, pinned via the `wasm-bindgen` submodule
  (`gbedford/mono-prototype`) and `[patch.crates-io]`: [#5375](https://github.com/wasm-bindgen/wasm-bindgen/pull/5375)
  (`reinit = "reconstruct"`, emitted by `#[durable_object]`; it also fixes a call-guard ordering
  bug in the existing reinit glue, a behavior change on its own), #5428 (`Option<T>` reads
  `null` as `None`), #5427 (`Option<T>` / `Nullable<T>` in generic positions), #5390 (js-sys
  element APIs as real generics under `js_sys_unstable_apis`), #5425 (`JsCanon`), #5426
  (`Promise<T, E>`), and the 20 generic-mono prototype commits still without a PR
  (`parity/mono-prototype-patches.md`, "After the session-16 restack"). wasm-bindgen 0.2.130 with these must ship before
  `ts-gen-worker` can publish. (`parity/errors.md` Session 7)

## Decisions taken

Creative choices made by agents, with justification. Guy reviews these.

- **Generated error enum is `JsException`, not `JsError`** (master): the idiomatic handler imports
  `ts_gen_worker::*` and `wasm_bindgen::prelude::*`, and `wasm_bindgen::prelude::JsError` makes
  `JsError` ambiguous (E0659). (`parity/errors.md`)
- **`--fallible` keeps getters/setters infallible; a getter `@throws {X}` is honored under both
  policies** since there is no `try_` form for getters. (`parity/errors.md`)
- **Single-type `@throws` routes through the enum when `--error-enum` is on**: one `?`-able type
  everywhere, per-method precision lives in the rendered `## Errors` doc. (`parity/errors.md`)
- **`Error` `Display` for `Js` is `Name: message`** (master): `worker::Error` renders that way and
  migrating specs assert on it (`sqlite.spec.ts`), so `respond_with_errors` can replace the
  port's hand-rolled handler. (`parity/crate.md`)
- **ts-gen-worker builds with plain `wasm-bindgen --target module`, no worker-build shim** (Guy).
  Trap recovery comes from wasm-bindgen's reinit; Durable Object instances surviving a reinit
  use the upstream `reinit = "reconstruct"` attribute (PR #5375), set by `#[durable_object]`.
- **ts-gen may emit non-extern inline wrappers** (master): it already emits factories, builders
  and `PartialEq` impls, so an inline `fn` that converts `f64`↔`u64` for `@rust u64`, or bakes a
  string literal into a literal-keyed overload (`kv.get_json(key)`), is within the same policy.
  Rule: wrappers are only emitted where the extern ABI cannot express the declared type; the
  extern binding stays available under a private name. Unblocks the `u64` and literal-overload
  items in `parity/workers-types.md`; Guy to confirm.
- **Members of parents beyond the first are flattened onto the child; earliest sibling parent
  wins a shared member** (`parity/workers-types.md`). **Types colliding with the Rust prelude are
  renamed** (`Result_` etc., no re-export).
- **Uninferrable method type parameters** (master, from workers-rs-3's design): a type parameter
  is kept when an argument mentions it or the return is exactly `T` / `T | undefined` /
  `T | null` (the turbofish is the cast); otherwise it is erased to the owning type's default
  (the type's default wins over the method's), else `JsValue`, and the parameter is dropped.
  `_promise` companions follow the same rule. Removes 11 of the port's 20 turbofishes.
- **Inline wrappers landed**: `@rust u64`/`i64` convert through a private `f64` extern;
  string-literal-keyed overloads get one wrapper per literal (`kv.get_json(key)`,
  `CompressionStream::new_gzip()`); `@throws {never}` on constructors honored.
  (`parity/errors.md` Session 5)
- **Awaited values lower like sync returns** (`text().await? -> String`, `Promise<number>` →
  `f64`); `declare const` statics are `SCREAMING_SNAKE_CASE`; instance members own the base name
  over statics (`_static` suffix); variadic callbacks lower to the untyped `Function`.
  (`parity/errors.md` Session 3)
- **`http` streaming round trip hands the original `ReadableStream` back**; `Headers ↔ HeaderMap`
  is fallible as in `worker`. Size and cold start vs `worker`: 6.89 MB vs 7.11 MB after
  `wasm-opt -Oz`, 168–194 ms vs 190–207 ms to first response. (`parity/crate.md` Session 4)
- **`Promise<T | null>` → `JsNullable<T>`; `_with_null` only where omission cannot express null;
  `<name>_promise()` beside every async binding** (workers-rs-2, `parity/errors.md` Session 2).
- **`http` feature mirrors `worker`'s**: `TryFrom` conversions and `#[event(fetch)]` accepting an
  `http::Request<Body>` handler; `test:ts-gen-worker-http` runs the suite in that mode.
  (`parity/crate.md`)
- **`--errors-as-error` emits `::js_sys::Error` fully qualified**, so an input declaring its own
  `Error` class cannot rebind it. (`parity/errors.md`)
