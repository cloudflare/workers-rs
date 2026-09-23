# Emscripten Tokio example

Stock Tokio on Workers. Building on the [emscripten example](../emscripten),
this enables the `worker` crate's `tokio` feature:

```toml
worker = { version = "...", features = ["tokio"] }
tokio = { version = "1", default-features = false, features = ["rt", "macros", "sync", "time"] }
```

With it, every `#[event]` and `#[durable_object]` handler runs on its own
Tokio event loop whose wait *is* the Workers event loop. There is no
`#[tokio::main]` and no runtime to build: `tokio::spawn`, `tokio::time`,
`tokio::sync` and `tokio::join!` work inside a plain `async fn` handler as they
do natively, with no stack switching.

```sh
npx wrangler dev
curl http://localhost:8787/spawn
curl http://localhost:8787/channels
```

| Route | Shows |
| --- | --- |
| `/spawn` | `tokio::spawn` and awaiting `JoinHandle`s |
| `/sleep?ms=100` | `tokio::time::sleep` |
| `/timeout` | `tokio::time::timeout` completing and elapsing |
| `/channels` | `mpsc` between spawned producers and the handler |
| `/mutex` | `tokio::sync::Mutex` shared across tasks |
| `/join` | `tokio::join!` over concurrent futures |

Each response carries the measured `elapsed_ms`, showing the work overlapping.

## Dependency patches

Tokio's host-driven event loop is pending upstream releases. The example's
`[patch.crates-io]` carries the branches; add the same block to your own
Worker:

| Crate | Source | Why |
| --- | --- | --- |
| tokio, tokio-macros | `guybedford/tokio` branch `emscripten-event-loop-host` | Host-driven event loops on emscripten (tokio-rs/tokio#8484) |
| mio | `guybedford/mio` branch `emscripten` | epoll selector on emscripten (tokio-rs/mio#1969) |
| libc | `rust-lang/libc` branch `libc-0.2` | emscripten epoll bindings, unreleased |
| wasm-bindgen | this repository's checkout | `#[wasm_bindgen(tokio)]` exports (wasm-bindgen/wasm-bindgen#5334) |
| wasm-streams | `guybedford/wasm-streams` branch `rlib-only` | cargo would otherwise link its `cdylib`, which emcc cannot produce |
