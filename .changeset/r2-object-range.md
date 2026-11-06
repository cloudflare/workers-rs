---
"workers-rs": patch
---

Fix `r2::Object::range()` panicking with `expected instance of R2Range` on every call; it now reads the runtime's plain `{ offset, length }` object and returns an error when the object was read without a range.
