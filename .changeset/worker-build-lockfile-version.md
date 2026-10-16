---
"workers-rs": patch
---

Make `worker-build` honour the `wasm-bindgen` version pinned by the root package in `Cargo.lock` when several versions are locked, instead of always taking the first (lowest) entry, and accept lockfile entries that carry a source suffix.
