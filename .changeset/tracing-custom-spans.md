---
"workers-rs": minor
---

Add `worker::observability`, bindings for Cloudflare Workers [custom spans](https://developers.cloudflare.com/workers/observability/traces/custom-spans/) (the `tracing` namespace of `cloudflare:workers`).

- `enter_span(name, |span| ...)` and `enter_span_async(name, |span| async { ... })` open a span that is active for the body and closes when it finishes, nesting under the automatic platform spans in the Workers Observability waterfall.
- `start_active_span(name, |span| ...)` keeps the span open after the body returns, and `start_span(name)` opens a span that is never active; both are closed with `Span::end`.
- `active_span()` returns the span active on the current async context, or the invocation's root span.
- `Span::set_attribute` (chainable), `Span::is_traced`, `Span::record_exception` / `record_js_exception`, `Span::set_status`, `Span::update_name`, and `Span::span_context` cover the rest of the runtime `Span` API.

The raw imports live in `worker_sys` (`Tracing`, `TracingSpan`). The new `custom-spans` example ships a `WorkersLayer` that bridges `tracing` span lifetimes onto the platform, so `span!` / `#[instrument]` get platform-measured durations with no Workers-specific code at the call site. Closes #899.
