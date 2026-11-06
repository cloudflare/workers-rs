---
"workers-rs": minor
---

Add `Request::signal()`, which returns the request's `AbortSignal`. For an incoming request it is aborted when the client disconnects, which requires the `enable_request_signal` compatibility flag.
