# `memory.discard`

Experimental support for the [WebAssembly memory-control proposal's
`memory.discard` instruction](https://github.com/WebAssembly/memory-control),
allowing a Rust worker to return freed memory pages to the operating system.

Without it, a wasm linear memory only ever grows: memory freed by the
allocator stays resident in the process for the lifetime of the isolate. With
`memory.discard`, freed pages are released back to the OS while remaining
addressable (they read back as zeroes on next use).

## Usage

Requires the `wasm_memory_discard` compatibility flag (experimental) in
workerd, and building with jemalloc as the global allocator via
[`jemallocator-discard`](https://crates.io/crates/jemallocator-discard):

```toml
# Cargo.toml
[dependencies]
jemallocator-discard = "0.7"
```

```rust
#[global_allocator]
static ALLOC: jemallocator_discard::Jemalloc = jemallocator_discard::Jemalloc;
```

```toml
# wrangler.toml
compatibility_flags = ["wasm_memory_discard"]

[build]
command = "cargo install \"worker-build@^0.8\" && WASM_BINDGEN_ARGS=--experimental-memory-discard worker-build --release --features jemalloc"
```

Building jemalloc for `wasm32-unknown-unknown` requires wasm libc headers:
set `EMSDK` (emsdk clang >= 20) or `JEMALLOC_WASM_SYSROOT`.

## How it works

- jemalloc is built for `wasm32-unknown-unknown` with `dirty_decay_ms:0`, so
  freed pages are immediately purged via `madvise(MADV_DONTNEED)`, which the
  wasm shim forwards to an `env.__wbindgen_memory_discard` function import.
- wasm-bindgen (`--experimental-memory-discard`, passed through worker-build
  via `WASM_BINDGEN_ARGS`) replaces the import with a generated local
  function whose body is a single `memory.discard` instruction, so page
  discard remains a pure wasm operation with no JS involved.
- The physical release is advisory: the runtime may rate limit the
  page-table work by declining a release, in which case the range is zeroed
  instead. Zero-readback is the only semantic guarantee of `memory.discard`;
  resident-memory reduction is best-effort.

## Benchmark

This example exposes a `/churn?mb=N` endpoint that allocates, touches and
frees N MB so resident set can be observed from outside. The bench script
builds two variants and measures workerd RSS across a 5 x 64MB churn
workload:

```sh
WORKERD=/path/to/workerd EMSDK=/path/to/emsdk \
  ./examples/memory-discard/bench/run.sh
```

- `discard` — jemalloc purging via `memory.discard`
- `dlmalloc` — the default Rust wasm allocator (status quo)

## Results

| variant | RSS baseline | RSS peak | RSS after churn | retained |
|---|---|---|---|---|
| jemalloc + memory.discard | 81MB | 146MB | **85MB** | **+4MB** |
| jemalloc + memory.fill | 89MB | 151MB | 151MB | +62MB |
| dlmalloc | 84MB | 147MB | 147MB | +63MB |

`memory.discard` returns essentially the entire churned working set to the
operating system, while both baselines retain it in full. The `memory.fill`
control was byte-for-byte identical to the discard build except for the
single instruction in the trampoline body (zeroing without page release),
isolating the effect of the page release itself.

Note that linear memory itself never shrinks, so address-space-derived
limits are unaffected — this is purely a resident-memory win.
