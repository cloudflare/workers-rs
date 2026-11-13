---
"workers-rs": minor
---

Expose native Container capabilities without application-side JavaScript reflection:

- `ContainerExecOptions::set_stdin(ReadableStream)` passes the original byte stream directly to `exec`; `set_stdin_pipe()` and `ExecProcess::stdin()` expose the platform's writable-pipe capability. Omitting stdin retains EOF behavior.
- `Container::named_image(name)` looks up a configured digest-pinned image without copying the image map.
- `Container::intercept_outbound_http(target, &Fetcher)` preserves the target, including a non-default port, passes the original handler capability, and awaits installation.
- `State::exported_fetcher(name, props)` and `Context::exported_fetcher(name, props)` create named loopback entrypoints with the original structured props. These require `enable_ctx_exports`. The factory is invoked with `Reflect.apply(factory, exports, [{props}])`, not an RPC object's remotely intercepted `call` or `apply` property.

Add synthetic WASM consumer regressions using real JavaScript byte streams and writable streams. They cover identity, incremental consumption before EOF, byte fidelity, stream errors, writable-pipe round trips, scoped fetch-capable factory results, factory receiver/call traps, named-image failures, and synchronous/asynchronous interception failures. They verify binding behavior, not deployed Cloudflare Container execution.

Sources checked 2026-11-13:
<https://developers.cloudflare.com/containers/api/durable-object-container/#exec>,
<https://developers.cloudflare.com/containers/api/durable-object-container/#images>,
<https://developers.cloudflare.com/containers/api/durable-object-container/#interceptoutboundhttp>,
<https://developers.cloudflare.com/workers/runtime-apis/context/#exports>, and
<https://developers.cloudflare.com/durable-objects/api/state/#exports>.

`ExecProcess::stdout()` and `stderr()` expose the original optional readable
capabilities as well, allowing callers to enforce their own consumption limits
without `output()` buffering. Do not call `output()` after consuming a stream.
