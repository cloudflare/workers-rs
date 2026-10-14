# ts-gen-worker parity tracking

Goal: `ts-gen-worker` reaches parity with the `worker` crate's existing surface
so that a `worker` user migrates with minor changes, and the `test/` suite runs
against it. New APIs are out of scope; only what `worker` exposes today.

Principles:

- The generated bindings are the API. Ergonomics are extra `impl` blocks on the
  generated types, never wrapper types mirroring them.
- Gaps are closed by ts-gen features (one commit per PR on the ts-gen branch
  `gbedford/workers-parity` in the `ts-gen` submodule) or by hand-written impls
  in this crate. Each choice is justified where it lands.
- Everything stays local. Nothing is pushed.

Per-topic notes live in `parity/*.md`; agents own one file each and this file
is the merged index.

## Status

| Area | State | Notes |
| --- | --- | --- |
| Error model (`--error-type`, `--fallible`, `@throws` enum) | integrated; plus `JsNullable` results, `_with_null` forms, `_promise` companions | `parity/errors.md` |
| Input from `@cloudflare/workers-types` | integrated: bindings generated from `node_modules/@cloudflare/workers-types/index.d.ts` + `types/annotations.d.ts` (`--annotate`: `@throws`, `@rust` integer hints, type-parameter defaults); `types/worker.d.ts` deleted; 19.8k generated lines | `parity/workers-types.md` |
| `Error`, `#[event]` macros, ergonomic impls | integrated; test worker migrated onto the layer (`#[event]`, `#[durable_object]`, shortcuts); `http` feature with `http::Request/Response` handlers, 176 tests green on both | `parity/crate.md` |
| Test suite port | integrated: 22/25 spec files, 161 tests green; `panic`, `signal`, `container` skipped | `parity/test-port.md` |

## Open problems for Guy

Items agents could not resolve without a decision. Each has a proposal. Details in the linked
file.

- **Uninferrable method type parameters** (`run::<JsValue>()`, `list::<JsValue>()` at every call).
  Options in `parity/workers-types.md`; a rule keyed on "T is the whole return vs nested" is
  proposed. Needs a call.
- **Multi-`extends` member reachability**: members of parents after the first are only reachable
  through `AsRef`. Proposal: flatten later parents' members into the child's extern block.
  (`parity/workers-types.md`)
- **How authoritative must `@throws` be?** Only Fetch-spec-pinned classes are annotated so far.
  Runtime-specific throws (KV/R2/D1, `DOMException` names) need a policy; a wrong annotation lands
  the value in `JsException::Other`, never a wrong variant. (`parity/errors.md`)
- **Overloads differing only in return type collapse to the first.** KV `get` variants, DO
  `storage.get(keys[])`. workers-rs-3 has a candidate fix (`ac26895`), pending integration.
  (`parity/test-port.md` #2)
- **`signal`/`panic` specs depend on worker-build shim state** (`is_registered`,
  `is_near_cpu_limit`, panic recovery). No binding corresponds; the shim story for ts-gen-worker
  is undecided. (`parity/test-port.md`)
- **`@throws {never}` on constructors is ignored** (constructors always catch). Proposal: honor it,
  since wasm-bindgen accepts a non-`catch` constructor. Low priority. (`parity/errors.md`)

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
- **ts-gen may emit non-extern inline wrappers** (master): it already emits factories, builders
  and `PartialEq` impls, so an inline `fn` that converts `f64`↔`u64` for `@rust u64`, or bakes a
  string literal into a literal-keyed overload (`kv.get_json(key)`), is within the same policy.
  Rule: wrappers are only emitted where the extern ABI cannot express the declared type; the
  extern binding stays available under a private name. Unblocks the `u64` and literal-overload
  items in `parity/workers-types.md`; Guy to confirm.
- **`Promise<T | null>` → `JsNullable<T>`; `_with_null` only where omission cannot express null;
  `<name>_promise()` beside every async binding** (workers-rs-2, `parity/errors.md` Session 2).
- **`http` feature mirrors `worker`'s**: `TryFrom` conversions and `#[event(fetch)]` accepting an
  `http::Request<Body>` handler; `test:ts-gen-worker-http` runs the suite in that mode.
  (`parity/crate.md`)
- **`--errors-as-error` emits `::js_sys::Error` fully qualified**, so an input declaring its own
  `Error` class cannot rebind it. (`parity/errors.md`)
