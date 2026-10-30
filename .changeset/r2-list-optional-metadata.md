---
"workers-rs": patch
---

Fix `r2::Object::http_metadata()` and `custom_metadata()` panicking on objects returned by `Bucket::list()` when the listing did not `include` that metadata; they now return empty metadata.
