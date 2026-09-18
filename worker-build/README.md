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
command = "cargo install -q worker-build && worker-build --emscripten --tokio --release"
```

With `--tokio`, the `#[event]` and `#[durable_object]` handlers are scheduled
on a Tokio event loop per invocation, whose wait *is* the host event loop, so
`tokio::net`, `tokio::time` and `tokio::spawn` work in plain async handlers
with no stack switching. Tokio comes from the branches listed in the
[emscripten-tcp example](../examples/emscripten-tcp). Hostname resolution
(`getaddrinfo`) is a blocking call with nothing to block on, so it fails with
`EAI_AGAIN`; connect to IP addresses. The mode is visible to crates as
`cfg(worker_tokio = "event_loop")`.

The build links a **bin** target rather than a `cdylib`: rustc drives `emcc`
as the linker, and `emcc` runs `wasm-bindgen` over the linked program as a
post-link step. Put handlers in `src/main.rs` with an empty `fn main() {}`,
or use `--bin NAME` when the package has several bin targets. Any `cdylib`
crate type in the package (or in a dependency) fails the link, since emcc
cannot produce one from a static Rust build.

On the first run worker-build downloads the pinned Emscripten SDK release into
its cache directory (`~/.cache/worker-build/emsdk-<version>`) and applies the
patches under `worker-build/patches/emscripten/` to the frontend. These are
backports the Rust link depends on that the pinned release does not yet
contain, taken from the current heads of the upstream pull requests:
marker-based `-sWASM_BINDGEN` (emscripten-core/emscripten#27208, released in
6.0.10), `emscripten_epoll_add_listener` readiness callbacks and timeout
keepalive release (#27547, #27720), hostname resolution under
`-sNODERAWSOCKETS` (#27693, #27742) and pending socket errors surfacing from
`recv` (#27724); each is removed as the pin moves past it.
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
the [emscripten-tcp example](../examples/emscripten-tcp) lists the
`[patch.crates-io]` entries a Worker currently adds for Tokio, mio, libc and
wasm-streams.
