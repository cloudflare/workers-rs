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

## External WebAssembly DWARF

To retain DWARF without embedding it in the runtime WebAssembly module, enable
external DWARF for the profile used by `worker-build`:

```toml
[package.metadata.wasm-pack.profile.release.wasm-bindgen]
split-debug-info = true
```

The setting defaults to `false`; set it explicitly to `false` to disable it.
The workers-rs project templates enable it for release builds by default.

`worker-build` keeps DWARF through `wasm-opt`, then emits this deterministic
artifact pair in the configured output directory:

- `index_bg.wasm` is the runtime module. Its `.debug_*` custom sections are
  removed.
- `index_bg.debug.wasm` is a complete, valid WebAssembly file containing the
  DWARF sections.

The runtime module contains one `external_debug_info` custom section. Its value
is the relative WebAssembly name `index_bg.debug.wasm`, following the
[WebAssembly external-DWARF convention][external-dwarf]. It is intentionally
not an absolute or deployment-specific URL, so moving the two files together
preserves the reference. A custom `--out-dir` changes their common directory,
not the relative reference.

This is external DWARF, not a JSON source map. The sidecar is not an importable
Worker module and must not be included by `find_additional_modules` or assigned
the `application/source-map` content type. Current Wrangler versions do not
automatically discover or upload it. Production symbolication therefore still
requires the downstream platform support described below.

### Wrangler integration contract

`worker-build` has no typed build-result or artifact API. Until one exists, the
smallest contract for Wrangler is the output pair itself and the standard
`external_debug_info` reference in the runtime module:

- Kind: external WebAssembly DWARF
- Runtime module: `index_bg.wasm`
- Relative reference: `index_bg.debug.wasm`
- Local artifact path: `<worker-build out-dir>/index_bg.debug.wasm`

Wrangler should parse the runtime custom section, resolve the relative name
against the runtime module's directory, and upload the resolved file as a
dedicated private debug artifact. No MIME type is specified here; the Workers
upload API must define its classification first.

The remaining cross-repository work is:

1. **workers-sdk/Wrangler:** discover `external_debug_info`, locate the relative
   sidecar, and upload it as a dedicated debug artifact rather than executable
   Worker code.
2. **Workers upload API:** define the artifact classification, private storage,
   separate size accounting, and Worker-version association for external Wasm
   DWARF.
3. **workerd:** compile the runtime Wasm with its canonical bundle URL through
   V8 `CompileOptions::source_url`.
4. **Inspector/observability:** resolve the relative sidecar from private
   version storage and provide it only to authenticated developer tooling and
   symbolication.

[external-dwarf]: https://github.com/WebAssembly/tool-conventions/blob/main/Dwarf.md#external-dwarf-file
