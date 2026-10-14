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
| `Error`, `#[event]` macros, ergonomic impls | integrated (`ext/`, `ts-gen-worker-macros`, `Router`, `#[durable_object]`); port adoption pending | `parity/crate.md` |
| Test suite port | integrated: 22/25 spec files, 161 tests green; `panic`, `signal`, `container` skipped | `parity/test-port.md` |

## Open problems for Guy

Items agents could not resolve without a decision. Each has a proposal. Details in the linked
file.

- **How authoritative must `@throws` be?** Only Fetch-spec-pinned classes are annotated so far.
  Runtime-specific throws (KV/R2/D1, `DOMException` names) need a policy; a wrong annotation lands
  the value in `JsException::Other`, never a wrong variant. (`parity/errors.md`)
- **`Promise<T | null>` lowers to `JsOption<T>`, which is empty only for `undefined`.** KV `get`,
  `Cache.match`, `getAlarm` resolve `null`, so `into_option()` gives `Some(null)`. Proposal: a
  nullish-aware wrapper in ts-gen (or `JsOption::is_empty` = null-or-undefined in wasm-bindgen).
  Port uses a local `util::nullable` meanwhile. (`parity/test-port.md` #1)
- **Async bindings have no sync `Promise`-returning form.** `async fn` imports call JS only when
  polled, so `blockConcurrencyWhile` in a DO constructor and `ctx.waitUntil(promise)` composition
  cannot start the call synchronously. Proposal: a `<name>_promise()` companion, or primary returns
  `Promise<T>` and relies on `IntoFuture`. (`parity/test-port.md` #3)
- **Overloads differing only in return type collapse to the first.** KV `get` variants, DO
  `storage.get(keys[])`. workers-rs-3 has a candidate fix (`ac26895`), pending integration.
  (`parity/test-port.md` #2)
- **`null` dropped in argument position** loses `new Response(null, init)`. Proposal: keep a
  `_with_null` form when more params follow. (`parity/test-port.md` #4)
- **`signal`/`panic` specs depend on worker-build shim state** (`is_registered`,
  `is_near_cpu_limit`, panic recovery). No binding corresponds; the shim story for ts-gen-worker
  is undecided. (`parity/test-port.md`)
- **Feature `http` differs from `worker`'s**: there it switches the fetch handler to
  `http::Request`/`Response`; here it only adds `HeaderMap` conversions. The `http` half of the
  test suite is blocked on porting those conversions. (`parity/crate.md`)
- **No guard that a `DurableObject` impl is exported** without `#[durable_object]`. Proposal: the
  hidden marker supertrait `worker` uses. (`parity/crate.md`)
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
- **`--errors-as-error` emits `::js_sys::Error` fully qualified**, so an input declaring its own
  `Error` class cannot rebind it. (`parity/errors.md`)
