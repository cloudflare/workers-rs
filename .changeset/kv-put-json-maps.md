---
"workers-rs": patch
---

Serialize `KvStore::put` values with `serde_json`, so `HashMap`, `BTreeMap` and `serde_json::Value::Object` fields are no longer stored as `{}` and `null` map entries are kept.
