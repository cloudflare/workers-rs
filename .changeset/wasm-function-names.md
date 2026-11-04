---
"workers-rs": minor
---

With `source-map = true`, `--emscripten` builds now raise the Cargo profile's `debug` to the line tables the source map needs when it is lower, so the profile no longer has to set `debug`. The module's function names go into the source map as `x_cloudflare_wasm_function_names`, and with `debug = 0`, the release default, the deployed module drops its name section, so it stays about the size of a build without a source map.
