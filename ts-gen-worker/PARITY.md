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
| Error model (`--error-type`, `--fallible`, `@throws` enum) | in progress | `parity/errors.md` |
| Input from `@cloudflare/workers-types` | in progress | `parity/workers-types.md` |
| `Error`, `#[event]` macros, ergonomic impls | in progress | `parity/crate.md` |
| Test suite port | in progress | `parity/test-port.md` |

## Open problems for Guy

Items agents could not resolve without a decision. Each has a proposal.

(none yet)

## Decisions taken

Creative choices made by agents, with justification. Guy reviews these.

(none yet)
