---
"workers-rs": patch
---

Fix `worker-build` failing with `invalid type: map, expected a string` when wasm-bindgen emits a `package.json` with a `dependencies` object (wasm-bindgen 0.2.94 and later), which happens whenever a JS snippet imports an NPM package.
