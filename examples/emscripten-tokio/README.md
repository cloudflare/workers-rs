# Emscripten Tokio example

> **Experimental Preview**

```sh
git clone https://github.com/cloudflare/workers-rs
cd workers-rs/examples/emscripten-tokio
npx wrangler dev
```

This uses an `experimental_tokio` [Tokio event loop on wasm-bindgen](https://wasm-bindgen.github.io/wasm-bindgen/reference/emscripten.html#tokio) to support Tokio inside of workers.

While upstream PRs are still in progress, a Tokio patchset is needed to run
this example, which is contained in `Cargo.toml`.

With these, every `#[event]` and `#[durable_object]` handler runs on its own
Tokio event loop whose wait *is* the Workers event loop. There is no
`#[tokio::main]` and no runtime to build: `tokio::spawn`, `tokio::time`,
`tokio::sync` and `tokio::join!` work inside a plain `async fn` handler as they
do natively, with no stack switching.

Worker-build also includes an Emscripten patch for a non-blocking epoll implementation
which is auto applied when worker-build installs Emscripten under `--emscripten`, and
passes the unstable cfgs both crates gate this behind (`tokio_unstable`, `wasm_bindgen_unstable_tokio`).

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

## What's Next

See the [emscripten-tcp](../emscripten-tcp/README.md) example for a full TCP
sockets app on workers with Emscripten and Tokio.
