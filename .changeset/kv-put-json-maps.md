---
"workers-rs": patch
---

Serialize `KvStore::put` values with maps as JS objects and bytes as arrays, so `HashMap`, `BTreeMap` and `serde_json::Value::Object` fields are no longer stored as `{}`.
