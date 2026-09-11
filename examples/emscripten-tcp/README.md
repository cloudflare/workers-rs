# Emscripten TCP example

A `worker-build --emscripten` Worker: stock `tokio::net::TcpStream` on
`wasm32-unknown-emscripten`, parking through JSPI on the host event loop.

```sh
curl 'http://localhost:8787/?host=example.com'
```

connects to port 80 of the host from inside the Worker and returns the HEAD
response. `fetch` is a plain synchronous Rust function: it builds a
current-thread Tokio runtime and `block_on`s the request; every blocking wait
suspends the Wasm stack via JSPI and resumes when the runtime delivers the
socket readiness, DNS result or timer.

## Layout

An emscripten build links a **bin** target: rustc drives `emcc` as the linker,
which runs `wasm-bindgen` as a post-link step. So handlers live in
`src/main.rs` with an empty `fn main() {}`, and there is no `cdylib`. The
`#[wasm_bindgen(jspi)]` export returns a Promise to the runtime while the Rust
side stays synchronous.

`rust-toolchain.toml` selects `beta` (`OwnedFd::try_clone` on emscripten,
used by mio's registry, lands in 1.99).

## Dependency patches

`wasm32-unknown-emscripten` support for the networking stack is pending
upstream releases. The example's `[patch.crates-io]` carries them; add the
same block to your own Worker:

| Crate | Source | Why |
| --- | --- | --- |
| tokio, tokio-macros | `guybedford/tokio` branch `emscripten-epoll` | JSPI parking and the `net` feature on emscripten (tokio-rs/tokio#8281 follow-ons) |
| mio | `guybedford/mio` rev `a62c9e4` | epoll selector on emscripten (tokio-rs/mio#1969) |
| libc | `rust-lang/libc` branch `libc-0.2` | emscripten epoll bindings, unreleased |
| wasm-streams | `guybedford/wasm-streams` branch `rlib-only` | rlib-only: cargo would otherwise link its cdylib, which emcc cannot produce |

## Run

```sh
npx wrangler dev
```

The first build downloads the Emscripten SDK into the worker-build cache. See
the [worker-build README](../../worker-build/README.md#emscripten) for the
toolchain details and overrides.
