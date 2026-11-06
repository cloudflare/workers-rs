---
"workers-rs": patch
---

Serialize queue messages sent with the default `json` content type JSON-compatibly, so `HashMap`, `BTreeMap` and `serde_json::Value::Object` fields no longer arrive at consumers as `{}` and `null` map entries are kept. `Option::None` fields of such messages now arrive as `null` instead of being omitted, as with `serde_json`.
