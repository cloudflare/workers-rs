# `memory.discard` experiment

Experiment for the [WebAssembly memory-control proposal's `memory.discard`
instruction](https://github.com/WebAssembly/memory-control), measuring
resident memory reduction for a Rust worker running on a `workerd` with the
`wasm_memory_discard` compatibility flag.

## How it works

- jemalloc is used as the global allocator (behind the opt-in `jemalloc`
  feature, since building it requires wasm libc headers), built for
  `wasm32-unknown-unknown` with `dirty_decay_ms:0`, so freed pages are
  immediately purged via `madvise(MADV_DONTNEED)`, which jemalloc's wasm
  shim forwards to an `env.__wbindgen_memory_discard` function import.
- wasm-bindgen (`--experimental-memory-discard`, passed through worker-build
  via `WASM_BINDGEN_ARGS`) replaces the import with a generated local
  function whose body is a single `memory.discard` instruction, so page
  discard remains a pure wasm operation with no JS involved.

Toolchain status:

- walrus — `memory.discard` support landed in 0.26.5
- workerd — landed behind the experimental `wasm_memory_discard`
  compatibility flag
- [wasm-bindgen#5287](https://github.com/wasm-bindgen/wasm-bindgen/pull/5287)
  — `--experimental-memory-discard` trampoline generation (the wasm-bindgen
  submodule tracks this branch)
- [jemallocator-discard](https://crates.io/crates/jemallocator-discard)
  — jemalloc 5.3 built for wasm32-unknown-unknown, purging through the
  `__wbindgen_memory_discard` import

## Running

Requires a `workerd` supporting the `wasm_memory_discard` compatibility
flag, and emsdk clang >= 20 for the jemalloc wasm build.

```sh
# from the repo root
chomp build   # builds wasm-bindgen CLI + worker-build

WORKERD=/path/to/workerd EMSDK=/path/to/emsdk \
  ./examples/memory-discard/bench/run.sh
```

The bench builds two variants and measures workerd RSS across a
5 x 64MB alloc/touch/free churn workload (`/churn?mb=64`):

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
