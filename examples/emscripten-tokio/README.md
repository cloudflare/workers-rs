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

## What this needs

Tokio's host-driven event loop is not yet released, so this example (and
[emscripten-tcp](../emscripten-tcp)) needs three things beyond the
[emscripten example](../emscripten):

1. **Tokio and mio from tagged forks.** The `[patch.crates-io]` block pins
   `guybedford/tokio` tag `1.53.1-cf.emscripten` (Tokio 1.53.1 plus
   tokio-rs/tokio#8484) and `guybedford/mio` tag `1.2.3-cf.emscripten` (mio
   1.2.3 plus tokio-rs/mio#1969), together with libc's unreleased emscripten
   epoll bindings. Copy the block into your own Worker.
2. **wasm-bindgen with `experimental_tokio` exports**
   (wasm-bindgen/wasm-bindgen#5334), which the `worker` crate's `tokio`
   feature emits on every handler; the block points at this repository's
   checkout.
3. **A patched Emscripten.** `emscripten_epoll_add_listener` (#27547) and
   `emscripten_dns_lookup_async` (#27742) are what the event loop is built on.
   worker-build applies them to the 6.0.10 SDK it installs, matching the
   `guybedford/emscripten` tag `6.0.10-cf.emscripten`; nothing to do unless
   you point `EMSCRIPTEN` at your own checkout, which then needs the same.

worker-build itself passes the unstable cfgs both crates gate this behind
(`tokio_unstable`, `wasm_bindgen_unstable_tokio`).
