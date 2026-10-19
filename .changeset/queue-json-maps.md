---
"workers-rs": patch
---

Serialize queue messages sent with the default `json` content type with maps as JS objects and bytes as arrays, so `HashMap`, `BTreeMap` and `serde_json::Value::Object` fields no longer arrive at consumers as `{}`.
