# Emscripten example

A Worker built with `worker-build --emscripten` for `wasm32-unknown-emscripten`.
The target brings Emscripten's libc and in-memory filesystem, so `std::fs`,
`std::io` and code written around files work unchanged.

The Worker is a word-frequency counter that works the way a command-line tool
would: the POSTed text is written to a file under `std::env::temp_dir()`, read
back line by line, and the report is written to a second file and returned
along with a directory listing.

```sh
npx wrangler dev
curl -X POST --data-binary @README.md http://localhost:8787/
```

## Layout

An emscripten build links a **bin** target: rustc drives `emcc` as the linker,
which runs `wasm-bindgen` as a post-link step. So handlers live in
`src/main.rs` with an empty `fn main() {}`, and there is no `cdylib`.

`rust-toolchain.toml` selects `beta`, and the `[patch.crates-io]` block points
`wasm-bindgen` at the checkout matching the CLI emcc runs (and takes an
rlib-only `wasm-streams`, since cargo would otherwise also link its `cdylib`).

## What this needs

Only `worker-build --emscripten`. Without Tokio there is no dependency on the
Tokio and mio forks or on the Emscripten patches that the
[emscripten-tokio](../emscripten-tokio) and [emscripten-tcp](../emscripten-tcp)
examples rely on: the stock Emscripten 6.0.10 release is enough. worker-build
installs that release into its cache on the first build (with the Tokio
patches applied, which are inert here). See the
[worker-build README](../../worker-build/README.md#emscripten) for the
toolchain details and overrides.
