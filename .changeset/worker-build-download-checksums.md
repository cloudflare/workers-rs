---
"workers-rs": patch
---

`worker-build` now checks the sha256 of the esbuild, wasm-opt and wasm-bindgen archives it downloads before extracting them, and fails the build on a mismatch. Hashes for the versions it uses are pinned in `worker-build/checksums.txt`; a wasm-bindgen version not listed there is checked against the `.sha256sum` file its release publishes. The Emscripten SDK is now fetched by commit instead of by tag.
