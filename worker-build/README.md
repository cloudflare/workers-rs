# worker-build

This is a tool to be used as a custom build command for a Cloudflare Workers project.

```toml
# wrangler.toml
# ...

[build]
command = "cargo install -q worker-build && worker-build --release"

[build.upload]
dir    = "build/worker"
format = "modules"
main   = "./shim.mjs"

[[build.upload.rules]]
globs = ["**/*.wasm"]
type  = "CompiledWasm"
```

## Environment Variables

You can override the default binary lookup/download behavior by setting these environment variables:

- **`WASM_BINDGEN_BIN`**: Path to a custom `wasm-bindgen` binary. When set, worker-build will use this binary instead of downloading or looking for a globally installed version.

- **`WASM_OPT_BIN`**: Path to a custom `wasm-opt` binary. When set, worker-build will use this binary instead of downloading one.

### Example

```bash
export WASM_BINDGEN_BIN=/path/to/custom/wasm-bindgen
export WASM_OPT_BIN=/path/to/custom/wasm-opt
worker-build --release
```
## Emscripten

`worker-build --emscripten` builds for `wasm32-unknown-emscripten`, giving
Workers a libc and epoll-backed sockets, so crates like stock Tokio `net` run
unmodified. The output has the same shape
as a regular build (`build/index.js` plus the Wasm), so `wrangler.toml` only
changes the build command:

```toml
main = "build/index.js"
compatibility_flags = ["nodejs_compat", "new_module_registry"]

[build]
command = "cargo install -q worker-build && worker-build --emscripten --release"
```

With the `worker` crate's `tokio` feature, the `#[event]` and
`#[durable_object]` handlers are scheduled on a Tokio event loop per
invocation, whose wait *is* the host event loop, so `tokio::net`,
`tokio::time` and `tokio::spawn` work in plain async handlers with no stack
switching. This is unstable end to end: it needs Tokio and mio from the
`guybedford` forks' `1.53.1-cf.emscripten` / `1.2.3-cf.emscripten` tags,
wasm-bindgen's `experimental_tokio` exports, and the Emscripten patches
described below; worker-build passes the `tokio_unstable` and
`wasm_bindgen_unstable_tokio` cfgs. Hostname resolution (`getaddrinfo`) is a
blocking call with nothing to block on, so it fails with `EAI_AGAIN`; connect
to IP addresses. A Worker without the feature needs none of this.

The [emscripten](../examples/emscripten), [emscripten-tokio](../examples/emscripten-tokio)
and [emscripten-tcp](../examples/emscripten-tcp) examples build up from
`std::fs` on the in-memory filesystem, through Tokio timers, tasks and
channels, to raw TCP; the tokio one lists the `[patch.crates-io]` block.

The build links a **bin** target rather than a `cdylib`: rustc drives `emcc`
as the linker, and `emcc` runs `wasm-bindgen` over the linked program as a
post-link step. Put handlers in `src/main.rs` with an empty `fn main() {}`,
or use `--bin NAME` when the package has several bin targets. Any `cdylib`
crate type in the package (or in a dependency) fails the link, since emcc
cannot produce one from a static Rust build.

On the first run worker-build downloads the pinned Emscripten SDK release into
its cache directory (`~/.cache/worker-build/emsdk-<version>`) and applies the
patches under `worker-build/patches/emscripten/` to the frontend. These are
the Emscripten changes Tokio's host-driven event loop depends on that are not
yet released: `emscripten_epoll_add_listener` readiness callbacks
(emscripten-core/emscripten#27547) and `emscripten_dns_lookup_async`
(#27742), as carried by the `guybedford/emscripten` tag
`6.0.10-cf.emscripten` (the 6.0.10 release plus those two pull requests).
Each is removed as the pin moves past it. A Worker that does not use Tokio
needs nothing from them; the patched frontend is a superset of the release.
Installing needs `python3` on `PATH`; the SDK ships its own LLVM, Binaryen and
Node.

Overrides for local toolchain development:

- **`EMSCRIPTEN`**: an emscripten frontend checkout (the directory holding
  `emcc`), used as-is without patching.
- **`EMSDK`**: an emsdk install providing the LLVM and Binaryen backend
  (`$EMSDK/upstream`) and Node.

Until wasm-bindgen 0.2.129, debuginfo builds (`--dev`, `--profiling`) need a
wasm-bindgen CLI with
[wasm-bindgen#5328](https://github.com/wasm-bindgen/wasm-bindgen/pull/5328),
whose DWARF output survives the exnref translation; build it from `main` and
point `WASM_BINDGEN_BIN` at it. `--release` builds work with the released CLI.

Emscripten networking support in the Rust ecosystem is still landing upstream;
the [emscripten-tokio example](../examples/emscripten-tokio) lists the
`[patch.crates-io]` entries a Worker using the `tokio` feature currently adds.
