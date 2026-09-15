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

When developing against the pinned wasm-bindgen submodule, build its CLI and
point worker-build at that exact binary so the CLI metadata schema matches the
patched Rust crates:

```bash
cargo +stable build --manifest-path wasm-bindgen/Cargo.toml -p wasm-bindgen-cli --bin wasm-bindgen
export WASM_BINDGEN_BIN="$PWD/wasm-bindgen/target/debug/wasm-bindgen"
worker-build --release
```

## Split WebAssembly debug information

To retain DWARF without embedding it in the runtime WebAssembly module, enable
debug splitting for the profile used by `worker-build`:

```toml
[package.metadata.wasm-pack.profile.release.wasm-bindgen]
split-debug-info = true
```

`worker-build` keeps debug information through `wasm-opt`, then writes the
optimized module to `index_bg.debug.wasm`. The runtime `index_bg.wasm` has its
`.debug_*` sections removed and points to the sidecar through an
`external_debug_info` custom section.
