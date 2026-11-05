---
"workers-rs": patch
---

`#[event(queue)]`, `#[event(email)]` and `#[event(connect)]` handlers can return any error
type convertible to `Box<dyn std::error::Error>` (e.g. `anyhow::Error` or a custom error), not
only `worker::Error`. Such an `Err` rejects with a JS `Error` carrying the source chain, while
types that implement `Into<JsValue>` keep their own conversion.
