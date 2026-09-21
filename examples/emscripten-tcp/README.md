# Emscripten TCP example

A `worker-build --emscripten` Worker with the `worker/tokio` feature: stock `tokio::net::TcpStream`
on `wasm32-unknown-emscripten`, driven by the host event loop.

```sh
curl 'http://localhost:8787/?host=1.1.1.1'
```

connects to port 80 of the host from inside the Worker and returns the HEAD
response; `/do?host=...` does the same from inside a Durable Object, and
`host=sleep` exercises a Tokio timer.

Each handler is scheduled on a Tokio event loop per invocation, whose wait is
the host event loop, so the plain async handler runs with no stack switching.
Hostname resolution has nothing to block on and fails with `EAI_AGAIN`; use IP
hosts.

## Layout

An emscripten build links a **bin** target: rustc drives `emcc` as the linker,
which runs `wasm-bindgen` as a post-link step. So handlers live in
`src/main.rs` with an empty `fn main() {}`, and there is no `cdylib`.

`rust-toolchain.toml` selects `beta` (`OwnedFd::try_clone` on emscripten,
used by mio's registry, lands in 1.99).

## Dependency patches

`wasm32-unknown-emscripten` support for the networking stack is pending
upstream releases. The example's `[patch.crates-io]` carries them; add the
same block to your own Worker:

| Crate | Source | Why |
| --- | --- | --- |
| tokio, tokio-macros | `guybedford/tokio` branch `emscripten-event-loop-host` | The `net` feature and host-driven event loops on emscripten (tokio-rs/tokio#8484) |
| mio | `guybedford/mio` branch `emscripten` | epoll selector on emscripten (tokio-rs/mio#1969) |
| libc | `rust-lang/libc` branch `libc-0.2` | emscripten epoll bindings, unreleased |
| wasm-streams | `guybedford/wasm-streams` branch `rlib-only` | rlib-only: cargo would otherwise link its cdylib, which emcc cannot produce |

## Run

```sh
npx wrangler dev
```

The first build downloads the Emscripten SDK into the worker-build cache. See
the [worker-build README](../../worker-build/README.md#emscripten) for the
toolchain details and overrides.
