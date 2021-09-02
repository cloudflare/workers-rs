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