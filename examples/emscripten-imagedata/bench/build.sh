#!/usr/bin/env bash
# Build both variants into build-simd/ and build-scalar/, keeping function
# names so SIMD usage can be attributed with wasm-dis.
set -euo pipefail
cd "$(dirname "$0")/.."
rm -rf build
RUSTFLAGS="-Ctarget-feature=+simd128 -Clink-arg=--profiling-funcs" EMCC_CFLAGS="-msimd128" \
  worker-build --emscripten --release
rm -rf build-simd && mv build build-simd
RUSTFLAGS="-Clink-arg=--profiling-funcs" worker-build --emscripten --release
rm -rf build-scalar && mv build build-scalar
ls -la build-*/index_bg.wasm
