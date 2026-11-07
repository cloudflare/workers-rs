# Emscripten Tokio example

> **Experimental Preview**

```sh
git clone https://github.com/cloudflare/workers-rs
cd workers-rs
cargo build --release -p worker-build
cd examples/emscripten-tokio
npx wrangler dev
```

The `[build]` command in `wrangler.toml` runs the `worker-build` built above.
To use a released `worker-build` instead, switch to the commented-out
production command.

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
```

Then try some of the example routes demonstrating Tokio functionality:

| Route | Shows |
| --- | --- |
| `curl http://localhost:8787/spawn` | `tokio::spawn` and awaiting `JoinHandle`s |
| `curl http://localhost:8787/sleep?ms=100` | `tokio::time::sleep` |
| `curl http://localhost:8787/timeout` | `tokio::time::timeout` completing and elapsing |
| `curl http://localhost:8787/channels` | `mpsc` between spawned producers and the handler |
| `curl http://localhost:8787/mutex` | `tokio::sync::Mutex` shared across tasks |
| `curl http://localhost:8787/join` | `tokio::join!` over concurrent futures |
| `curl http://localhost:8787/stack` | a spawned task using 256 KiB of stack |

Each response carries the measured `elapsed_ms`, showing the work overlapping.

## Workflows

A `#[workflow]` gets Tokio too. `run` has its own event loop like an
`#[event]` handler, and so does each step attempt (every retry included) and
each rollback call, so steps can spawn tasks and use Tokio timers and I/O. A
Tokio resource such as a connection or a spawned task belongs to the attempt
that created it and shouldn't be kept for a later step.

| Route | Shows |
| --- | --- |
| `curl -X POST http://localhost:8787/workflow` | start an instance whose steps use `tokio::spawn` and `tokio::time`; responds with its id |
| `curl http://localhost:8787/workflow/<id>` | the instance's status and output |
| `curl -X POST 'http://localhost:8787/workflow?fail'` | an instance that fails after its `reserve` step, so that step's rollback runs |
| `curl http://localhost:8787/workflow/<id>/rollback` | what the rollback handler recorded |

## What's Next

See the [emscripten-tcp](../emscripten-tcp/README.md) example for a full TCP
sockets app on workers with Emscripten and Tokio.
