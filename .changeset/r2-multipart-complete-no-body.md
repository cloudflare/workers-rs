---
"workers-rs": patch
---

`MultipartUpload::complete` now returns an `Object` without a body. Before, `body()` returned `Some` for an object that has none, and reading it failed.
