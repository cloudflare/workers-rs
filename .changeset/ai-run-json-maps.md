---
"workers-rs": patch
---

Serialize `Ai::run` and `Ai::run_bytes` inputs as JSON-compatible values, so maps such as tool `parameters` are no longer sent as `{}`. `Option::None` fields are now sent as `null`; use `#[serde(skip_serializing_if = "Option::is_none")]` to omit them.
