# Emscripten incoming TCP example

> **Experimental Preview**

```sh
git clone https://github.com/cloudflare/workers-rs
cd workers-rs/examples/emscripten-incoming-tcp
npx wrangler dev
```

An inbound TCP server with `tokio::net::TcpListener`, using the Emscripten
Tokio patchset (the `guybedford/tokio` tag pinned in `Cargo.toml`,
tokio-rs/tokio#8438). Building on the [emscripten-tcp example](../emscripten-tcp),
a Tokio line-echo server greets each connection, echoes every line back
upper-cased and hangs up on `quit`.

```sh
nc 127.0.0.1 7000
```

Inbound TCP on port 7000 (`[[connect]]` in `wrangler.toml`) reaches the
Worker's `connect` handler, which on the first connection binds a
`TcpListener` on that port and spawns its accept loop, and hands every
connection to the listener with `Socket::handle_as_node_connection`. The
accept loop then serves connections like any Tokio server. The handler
awaits the hand-off to completion: its return closes the socket, so
spawning it or returning early would drop the connection.

## Two runtimes

The handler mirrors how Workers themselves run. A Worker's top level does no
I/O; each request does its own, inside its own I/O context. So the `connect`
export runs on the thread's shared (ambient) Tokio event loop, which hosts
only the listener and the accept loop, and each accepted connection is moved
onto an event loop of its own with `wasm_bindgen_futures::tokio::schedule_isolated`,
created inside the `connect` invocation that delivered it: all of that
connection's I/O runs within its own request. `#[event(connect)]` gives a
handler such a per-invocation runtime directly; the raw
`#[wasm_bindgen(experimental_tokio)]` export here is the shared one the
listener needs.
