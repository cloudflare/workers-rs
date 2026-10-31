---
"workers-rs": minor
---

Add `Storage::get_with_options` and `Storage::get_multiple_with_options` for Durable Objects,
taking a `GetOptions` with the runtime's `DurableObjectGetOptions` fields (`allowConcurrency`,
`noCache`). The existing `get` and `get_multiple` are unchanged.
