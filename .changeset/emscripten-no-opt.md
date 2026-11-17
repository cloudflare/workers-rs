---
"workers-rs": patch
---

`--no-opt` now applies to `--emscripten` builds: the emcc link runs at `-O0`, so Binaryen skips its optimization passes. wasm-opt still runs once at link time for the `--translate-to-exnref` pass these builds need.
