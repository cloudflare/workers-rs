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

## Smaller release builds without Wasm names

Release builds keep Wasm function names, which CPU profiles and stack traces show. Explicit
`wasm-opt` arguments are passed through as written, plus only the `--enable-*` features the
Workers runtime needs, so they don't keep names unless they include `-g`:

```toml
[package.metadata.wasm-pack.profile.release]
# names dropped
wasm-opt = ["-O"]
# names kept
# wasm-opt = ["-O", "-g"]
```

Use this instead of `strip = true` in the Cargo profile, which makes the linker drop every custom
section that later steps and tools rely on.

## Environment Variables

You can override the default binary lookup/download behavior by setting these environment variables:

- **`WASM_BINDGEN_BIN`**: Path to a custom `wasm-bindgen` binary. When set, worker-build will use this binary instead of downloading or looking for a globally installed version.

- **`WASM_OPT_BIN`**: Path to a custom `wasm-opt` binary. When set, worker-build will use this binary instead of downloading one.

- **`EMSCRIPTEN`**: Under `--emscripten`, an emscripten frontend checkout (the directory holding
  `emcc`), used as-is without patching.
- **`EMSDK`**: Under `--emscripten`, an emsdk install providing the LLVM and Binaryen backend
  (`$EMSDK/upstream`) and Node.

### Example

```bash
export WASM_BINDGEN_BIN=/path/to/custom/wasm-bindgen
export WASM_OPT_BIN=/path/to/custom/wasm-opt
worker-build --release
```

