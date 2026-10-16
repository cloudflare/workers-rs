# workers-rs Benchmark Suite

The same routes implemented three ways, measured side by side under Miniflare:

* `js/`: a hand-written `export default { fetch }`, the floor.
* `src/`: the `worker` crate, packaged by `worker-build`.
* `../ts-gen-worker/benchmark/`: the `ts-gen-worker` crate.

A fourth variant, `worker (no shim)`, serves the `worker` build's wasm and glue without
`worker-build`'s entrypoint shim, to separate the shim's cost from the bindings'.

## How to run

Clone workers-rs with all submodules, then from the root of workers-rs:

```bash
npm run build      # the local wasm-bindgen and worker-build
npm run bench
```

or `chomp bench`. `build.sh` builds both Rust workers in release mode with the same
wasm-bindgen and the same `wasm-opt -Oz` pass.

## Routes

| route | work |
| --- | --- |
| `/hello` | `new Response("hello")` |
| `/headers` | read 5 request headers, set 5 response headers |
| `/echo` | POST 16 KB, `text()`, respond with it |
| `/json` | POST ~2 KB of JSON, parse to a struct, increment a field, respond JSON |
| `/kv` | KV get and put of a 1 KB value |
| `/stream` | 1 MB streamed in 8 KB chunks |
| `/fanout` | 10 parallel sub-requests to `/hello` through a service binding to itself |

The `worker` crate also keeps `/benchmark`: 10 parallel sub-requests to `/stream` through
the global `fetch`.

## Method

`run.js` starts one Miniflare per variant with identical options and sends requests with
`dispatchFetch`, timing each with `performance.now()` from dispatch to the last body byte.
Variants are interleaved request by request so drift on the machine affects all of them
alike. Each repetition waits for a 1-minute load average below 2, measures cold start on
fresh instances, warms up, then per route discards the first 50 requests per variant and
reports p50/p90/p99 and sequential requests/sec over the next 2000. Flags: `--requests`,
`--warmup`, `--reps`, `--cold`, `--prewarm`, `--routes hello,json`, `--out results.json`.

## Rust Toolchain

`rust-toolchain.toml` in the root of workers-rs sets the Rust toolchain. Changing this can be used to
benchmark against different toolchain versions.
