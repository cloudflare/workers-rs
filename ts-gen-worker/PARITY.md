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
| Error model (`--error-type`, `--fallible`, `@throws` enum) | integrated | `parity/errors.md` |
| Input from `@cloudflare/workers-types` | in progress | `parity/workers-types.md` |
| `Error`, `#[event]` macros, ergonomic impls | in progress | `parity/crate.md` |
| Test suite port | in progress | `parity/test-port.md` |

## Open problems for Guy

Items agents could not resolve without a decision. Each has a proposal. Details in the linked
file.

- **How authoritative must `@throws` be?** Only Fetch-spec-pinned classes are annotated so far.
  Runtime-specific throws (KV/R2/D1, `DOMException` names) need a policy; a wrong annotation lands
  the value in `JsException::Other`, never a wrong variant. (`parity/errors.md`)
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
- **`--errors-as-error` emits `::js_sys::Error` fully qualified**, so an input declaring its own
  `Error` class cannot rebind it. (`parity/errors.md`)
