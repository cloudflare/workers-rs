# Emscripten TCP example

> **Experimental Preview**

```sh
git clone https://github.com/cloudflare/workers-rs
cd workers-rs/examples/emscripten-tcp
npx wrangler dev
```

Raw TCP from a Worker with `tokio::net`, using the Emscripten Tokio patchset
(the `guybedford/tokio` tag pinned in `Cargo.toml`, tokio-rs/tokio#8438).
Building on the [emscripten-tokio example](../emscripten-tokio), the Worker
connects to port 80 of a host, sends an HTTP `HEAD` request and returns the
reply; `/do` does the same from inside a Durable Object.

```sh
npx wrangler dev
curl 'http://localhost:8787/?host=example.com'
curl 'http://localhost:8787/do?host=1.1.1.1'
```

`TcpStream::connect((host, 80))`, `write_all` and `read_to_string` are the
ordinary Tokio API, under a `tokio::time::timeout`. The socket is backed by
the Workers `connect()` API through Emscripten's `-sNODERAWSOCKETS`, and
readiness flows through the same event loop that drives the handler. The
hostname resolves through Emscripten's asynchronous `getaddrinfo`
([emscripten#27742](https://github.com/emscripten-core/emscripten/pull/27742),
applied by worker-build), which Tokio's `ToSocketAddrs` uses on this target.
