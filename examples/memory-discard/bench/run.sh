#!/usr/bin/env bash
# memory.discard bench: builds the worker in two variants and measures
# workerd RSS across an alloc/free churn workload.
#
# Requires:
#   - workerd supporting the wasm_memory_discard compatibility flag (WORKERD env)
#   - emsdk clang >= 20 for the jemalloc wasm build (EMSDK env)
#   - wasm-bindgen CLI built from the workers-rs wasm-bindgen submodule
set -euo pipefail
cd "$(dirname "$0")/.."

WORKERD=${WORKERD:-workerd}
export EMSDK=${EMSDK:-$HOME/Projects/emsdk}
export CC_wasm32_unknown_unknown=${CC_wasm32_unknown_unknown:-$EMSDK/upstream/bin/clang}
WASM_BINDGEN_BIN_DEFAULT=$(pwd)/../../wasm-bindgen/target/debug/wasm-bindgen
export WASM_BINDGEN_BIN=${WASM_BINDGEN_BIN:-$WASM_BINDGEN_BIN_DEFAULT}
WORKER_BUILD=${WORKER_BUILD:-$(pwd)/../../target/debug/worker-build}
PORT=${PORT:-8123}
CHURN_MB=${CHURN_MB:-64}
ITERS=${ITERS:-5}

build_variant() {
  local name=$1
  shift
  echo "== building variant: $name" >&2
  rm -rf build
  "$@" >&2
  rm -rf "build-$name"
  cp -r build "build-$name"
}

rss_kb() {
  awk '/VmRSS/{print $2}' "/proc/$1/status"
}

hwm_kb() {
  awk '/VmHWM/{print $2}' "/proc/$1/status"
}

measure_variant() {
  local name=$1
  local config="bench/config-$name.capnp"
  cat >"$config" <<EOF
using Workerd = import "/workerd/workerd.capnp";

const config :Workerd.Config = (
  services = [ (name = "main", worker = .mainWorker) ],
  sockets = [ (name = "http", address = "127.0.0.1:$PORT", http = (), service = "main") ]
);

const mainWorker :Workerd.Worker = (
  modules = [
    (name = "index.js", esModule = embed "../build-$name/index.js"),
    (name = "index_bg.wasm", wasm = embed "../build-$name/index_bg.wasm"),
  ],
  compatibilityDate = "2025-05-01",
  compatibilityFlags = ["wasm_memory_discard"],
);
EOF

  "$WORKERD" serve --experimental "$config" &
  local pid=$!
  trap "kill $pid 2>/dev/null || true" RETURN

  for _ in $(seq 1 100); do
    curl -sf "http://127.0.0.1:$PORT/mem" >/dev/null 2>&1 && break
    sleep 0.1
  done

  # Warmup and settle
  curl -sf "http://127.0.0.1:$PORT/churn?mb=8" >/dev/null
  sleep 0.5
  local rss_base=$(rss_kb $pid)

  local churn_out=""
  for _ in $(seq 1 "$ITERS"); do
    churn_out=$(curl -sf "http://127.0.0.1:$PORT/churn?mb=$CHURN_MB")
  done
  sleep 0.5
  local rss_after=$(rss_kb $pid)
  local hwm=$(hwm_kb $pid)
  local linear=$(curl -sf "http://127.0.0.1:$PORT/mem")

  kill $pid 2>/dev/null || true
  wait $pid 2>/dev/null || true

  echo "$name: rss_baseline=$((rss_base / 1024))MB rss_after_churn=$((rss_after / 1024))MB rss_peak=$((hwm / 1024))MB retained_delta=$(((rss_after - rss_base) / 1024))MB"
  echo "  last churn: $churn_out"
  echo "  linear: $linear"
}

echo "workerd: $WORKERD"
echo "workload: $ITERS x ${CHURN_MB}MB churn (64KB chunks)"
echo

build_variant discard env WASM_BINDGEN_ARGS=--experimental-memory-discard "$WORKER_BUILD" --release --features jemalloc
build_variant dlmalloc "$WORKER_BUILD" --release

measure_variant discard
measure_variant dlmalloc
