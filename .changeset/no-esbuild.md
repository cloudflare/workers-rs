---
"workers-rs": minor
---

`worker-build` no longer bundles its output with esbuild. `build/index.js` is now the entry of a module graph, importing `build/index_bg.js` and any wasm-bindgen `build/snippets/`, for the consumer to bundle. Wrangler does this by default; with `--no-bundle` or an explicit Miniflare manifest the extra modules must be included. Minification is left to the consumer, so `NO_MINIFY` no longer has any effect.
