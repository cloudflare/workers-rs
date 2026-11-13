---
"workers-rs": patch
---

Serialize `Container::start` and `Container::exec` options as plain JavaScript
objects, converting their environment maps to `Record<string, string>` at the
binding boundary without changing the Rust options API. Preserve explicitly
set startup fields and serialize startup labels as a record as well.

Previously the calls passed wasm-bindgen-exported Rust option classes whose
`env` getters returned JavaScript `Map` instances. The native Container API
expects record properties and did not pass those entries to the process.
The unused startup object conversion also incorrectly removed populated
entrypoint, Internet, and environment fields.

Add WASM record-shape tests for startup and exec options.

Source: <https://developers.cloudflare.com/containers/api/durable-object-container/#start>
and <https://developers.cloudflare.com/containers/api/durable-object-container/#exec>,
checked 2026-11-12. Both environment options are `Record<string, string>`;
exec inherits only PATH from startup, not the other startup variables.
