---
"workers-rs": minor
---

`worker-build` now bundles the worker shim with rolldown, linked in as a library, instead of
downloading an esbuild binary on first build. The `ESBUILD_BIN` override no longer applies.
Building `worker-build` now requires Rust 1.97, and the workspace MSRV moves to 1.97 to match.
