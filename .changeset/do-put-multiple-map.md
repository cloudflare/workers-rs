---
"workers-rs": patch
---

Durable Object `Storage::put_multiple` and `Transaction::put_multiple` now store the entries of a map (`HashMap`, `BTreeMap`, `serde_json::Map`). Before, a map was serialized to a JavaScript `Map`, so the call returned `Ok` and stored nothing.
