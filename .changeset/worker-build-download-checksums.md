---
"workers-rs": patch
---

`worker-build` now checks the sha256 of the esbuild, wasm-opt and wasm-bindgen archives it downloads before extracting them, and fails the build on a mismatch or when there is no sha256 to check. esbuild and wasm-opt are checked against hashes pinned in `worker-build/checksums.txt`; wasm-bindgen is checked against the `.sha256sum` file its release publishes. The Emscripten SDK is now fetched by commit instead of by tag.
