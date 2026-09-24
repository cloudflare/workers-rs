# Emscripten TCP example

Raw TCP from a Worker with stock `tokio::net`. Building on the
[emscripten-tokio example](../emscripten-tokio), the Worker is a small TCP
proxy: a POST body is written to the target address and the reply returned,
a GET sends an HTTP `HEAD` request, and `/do` does the same from inside a
Durable Object.

```sh
npx wrangler dev
curl 'http://localhost:8787/?host=1.1.1.1'
curl -X POST --data-binary $'GET / HTTP/1.0\r\nHost: 1.1.1.1\r\n\r\n' 'http://localhost:8787/?host=1.1.1.1&port=80'
curl 'http://localhost:8787/do?host=1.1.1.1'
```

`TcpStream::connect`, `write_all` and `read_to_end` are the ordinary Tokio
calls, under a `tokio::time::timeout`. The socket is backed by the Workers
`connect()` API through Emscripten's `-sNODERAWSOCKETS`, and readiness flows
through the same event loop that drives the handler.

Hostname resolution has nothing to block on and fails with `EAI_AGAIN`; use IP
hosts.

## What this needs

The same as the [emscripten-tokio example](../emscripten-tokio#what-this-needs):
the tagged Tokio and mio forks, wasm-bindgen's `experimental_tokio` exports,
and the Emscripten patches worker-build applies. The `tokio` `net` and
`io-util` features join the ones enabled there.
