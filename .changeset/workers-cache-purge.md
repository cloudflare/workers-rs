---
"workers-rs": minor
---

Add bindings for Workers Cache purging through `Context::cache()` and
`worker::cache()`. `CachePurgeOptions` supports tags, path prefixes, and purging
everything, with incompatible options rejected before calling the runtime.
`CachePurgeResult` exposes the runtime's success status and structured errors.
