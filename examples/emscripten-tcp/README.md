# Emscripten TCP example

A `worker-build --emscripten` Worker: stock `tokio::net::TcpStream` on
`wasm32-unknown-emscripten`, parking through JSPI on the host event loop.

```sh
curl 'http://localhost:8787/?host=example.com'
```

connects to port 80 of the host from inside the Worker and returns the HEAD
response; `/do?host=...` does the same from inside a Durable Object, and
`host=sleep` exercises a Tokio timer.

The build command selects the Tokio integration:

* `worker-build --emscripten --tokio=jspi` (the default here): each handler is
  a `#[wasm_bindgen(jspi)]` export on its own fiber and blocks on a
  current-thread runtime; every blocking wait suspends the Wasm stack and
  resumes when the host delivers socket readiness, a DNS result or a timer.
* `worker-build --emscripten --tokio`: the handler is scheduled on a Tokio
  event-loop runtime per invocation, whose wait is the host event loop, so the
  plain async handler runs with no stack switching. Hostname resolution still
  needs JSPI; use IP hosts in this mode.

`head` in `src/main.rs` shows the one difference visible to the handler,
under `cfg(worker_tokio = "jspi")`.

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
| tokio, tokio-macros | `guybedford/tokio` branch `emscripten-tokio` | The `net` feature, JSPI parking with fiber-owned runtime context, and the `EventLoopRuntime` on emscripten (tokio-rs/tokio#8281, #8479 follow-ons) |
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
