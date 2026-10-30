#!/usr/bin/env bash
# Builds the Rust benchmark workers through one pipeline: the workspace release profile,
# the submodule's wasm-bindgen, and the same `wasm-opt -Oz` pass with the feature set
# worker-build enables, so the variants differ only in the crate they are written against.
set -euo pipefail
cd "$(dirname "$0")/.."

WB=$PWD/wasm-bindgen/target/debug/wasm-bindgen
OPT=${WASM_OPT:-$(ls ~/.cache/worker-build/binaryen-*/bin/wasm-opt | head -n1)}
FEATURES=(--enable-mutable-globals --enable-sign-ext --enable-nontrapping-float-to-int
  --enable-bulk-memory --enable-multivalue --enable-reference-types --enable-exception-handling)

oz() {
  "$OPT" -Oz "${FEATURES[@]}" -o "$1" "$1"
}

# worker, packaged by worker-build as a user ships it (entrypoint shim + glue); `--no-opt`
# because its own `wasm-opt -O` pass is replaced by the shared one.
(cd benchmark && NO_MINIFY=1 WASM_BINDGEN_BIN=$WB ../target/debug/worker-build --release --no-opt)
oz benchmark/build/index_bg.wasm

# The same wasm and glue with worker-build's flags but without its shim, as a control for
# what the shim costs per request.
rm -rf benchmark/build-noshim
"$WB" --target module --out-name index --no-typescript --experimental-reset-state-function \
  --force-enable-abort-handler --out-dir benchmark/build-noshim \
  target/wasm32-unknown-unknown/release/workers_rs_benchmark.wasm
oz benchmark/build-noshim/index_bg.wasm
cp benchmark/noshim.js benchmark/build-noshim/entry.js

# ts-gen-worker, built as ts-gen-worker/test is (chompfile.toml: js-sys's unstable API, own
# target dir).
RUSTFLAGS='--cfg getrandom_backend="wasm_js" --cfg js_sys_unstable_apis' CARGO_TARGET_DIR=target/mono \
  cargo build --release --target wasm32-unknown-unknown -p ts-gen-worker-benchmark
rm -rf ts-gen-worker/benchmark/build
"$WB" --target module --out-name index --force-enable-abort-handler \
  --out-dir ts-gen-worker/benchmark/build \
  target/mono/wasm32-unknown-unknown/release/ts_gen_worker_benchmark.wasm
oz ts-gen-worker/benchmark/build/index_bg.wasm
