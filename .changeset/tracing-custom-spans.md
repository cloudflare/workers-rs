---
"workers-rs": minor
---

Add `worker::observability` — bindings for Cloudflare Workers [custom spans](https://developers.cloudflare.com/changelog/post/2026-06-16-custom-spans/) (`cloudflare:workers` `enterSpan` / `startActiveSpan`).

- `enter_span(name, |span| ...)` and `enter_span_async(name, |span| async { ... })` open callback-scoped custom trace spans that nest under the automatic platform spans in the Workers Observability waterfall.
- `start_active_span(name)` opens a span that outlives the callback and is closed by `Span::end` — for streams, and for bridging `tracing`'s separate span create/close.
- `Span::set_attribute` / `Span::is_traced` attach metadata and check sampling.
- `with_active_span` exposes the innermost open span so a `tracing_subscriber::Layer` can forward `tracing` events/fields onto it.

The new `custom-spans` example ships a `WorkersLayer` that bridges `tracing` span *lifetimes* onto the platform, so `span!` / `#[instrument]` get platform-measured durations with no Workers-specific code at the call site. Addresses #899.
