---
"workers-rs": minor
---

Add `Storage::get_with_options` and `Storage::get_multiple_with_options` for Durable Objects,
taking `DurableObjectGetOptions` (`allowConcurrency`, `noCache`). The existing `get` and
`get_multiple` are unchanged.
