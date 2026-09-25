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
a Tokio line-echo server runs inside a Durable Object: it greets each
connection, echoes every line back upper-cased and hangs up on `quit`.

```sh
nc 127.0.0.1 7000
```

Inbound TCP on port 7000 (`[[connect]]` in `wrangler.toml`) reaches the
Worker's `#[event(connect)]` handler, which opens a connection to the object
with `Stub::connect` and pipes the two sockets together. The object's
`#[durable_object(connect)]` handler binds a `TcpListener` on the same port on
the first connection and hands every connection to it with
`Socket::handle_as_node_connection`, so the listener's `accept` loop serves
them like any Tokio server.

## Why the Durable Object

A Durable Object is one I/O context, so its handlers share one Tokio runtime
(the thread's ambient event loop) and a listener outlives any single
connection. A Worker's handlers each run on a runtime of their own for the
duration of that request, which is right for serving the connection in hand
but cannot host a listener that other requests' connections arrive on.

## What this needs

The same as the [emscripten-tokio example](../emscripten-tokio#what-this-needs).
