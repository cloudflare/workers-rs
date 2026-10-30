---
"workers-rs": patch
---

Default `--emscripten` builds to a 1MB stack, matching `wasm32-unknown-unknown` builds. Emscripten's 64KB default overflowed in debug builds of larger async code (e.g. a SQLx `PgPool`), surfacing as `memory access out of bounds`.
