---
"workers-rs": minor
---

Add `worker::Method::Query` for HTTP QUERY (RFC 10008), preserving the method on incoming and outgoing requests instead of interpreting it as GET. `Method::all()` and catch-all router handlers now include QUERY. Downstream exhaustive matches on `Method` must handle the new variant.
