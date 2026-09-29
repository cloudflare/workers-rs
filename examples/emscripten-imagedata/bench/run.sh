#!/usr/bin/env bash
# Serve a built variant with wrangler and run bench.mjs against it.
#   bench/run.sh simd|scalar [repeat] [samples]
set -euo pipefail
cd "$(dirname "$0")/.."
ln -sfn "build-$1" build
npx wrangler dev -c bench/wrangler.bench.toml --port 8799 > /tmp/wrangler-bench.log 2>&1 &
trap 'kill %1 2>/dev/null' EXIT
for _ in $(seq 50); do curl -sf localhost:8799/ > /dev/null && break; sleep 0.2; done
echo "== $1"
node bench/bench.mjs http://localhost:8799 "${2:-10}" "${3:-5}"
