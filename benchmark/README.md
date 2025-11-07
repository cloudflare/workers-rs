# workers-rs Benchmark Suite

Performance benchmark for workers-rs that measures streaming and parallel sub-request performance.

## How to run

First, make sure to clone workers-rs with all submodules.

Then from the root of workers-rs:

```bash
npm run build
```

to build the local `worker-build`.

Then run the benchmark:

```bash
cd benchmark
npm install
npm run bench
```

## What it does

- Streams 1MB of data from `/stream` endpoint in 8KB chunks
- Makes 10 parallel sub-requests to `/stream` from `/benchmark` endpoint
- All requests are internal (no network I/O) to isolate workers-rs performance
- Runs 20 iterations with 3 warmup requests

## Output

The benchmark provides:

- Per-iteration timing for Node.js end-to-end and Worker internal execution
- Summary statistics: average, min, and max times
- Data transfer statistics (10MB per iteration = 10 parallel 1MB streams)
- Average throughput in Mbps

## Configuration

Adjust parameters in `run.mjs`:
- `iterations` - Number of benchmark runs (default: 20)
- Warmup count (default: 3)

Adjust workload in `src/lib.rs`:
- Number of parallel requests (default: 10)
- Data size per request (default: 1MB)
- Chunk size for streaming (default: 8KB)

## Rust Toolchain

`rust-toolchain.toml` in the root of workers-rs sets the Rust toolchain. Changing this can be used to
benchmark against different toolchain versions.

## Compatibility Date

The current compaitibility date is set to `2025-11-01` in the `wrangler.toml`. Finalization registry was enabled as of `2025-05-05`, so is included.