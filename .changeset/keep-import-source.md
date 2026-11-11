---
"workers-rs": minor
---

`worker-build` no longer rewrites the Wasm `import source` in `build/index.js` to a plain `import`, now that workerd and Wrangler support source phase imports. Needs a Wrangler version whose esbuild parses `import source`.
