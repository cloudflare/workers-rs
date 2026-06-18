---
"workers-rs": minor
---

Add `worker::observability` — bindings for Cloudflare Workers [custom spans](https://developers.cloudflare.com/changelog/post/2026-06-16-custom-spans/) (`cloudflare:workers` `enterSpan`).

- `enter_span(name, |span| ...)` and `enter_span_async(name, |span| async { ... })` open custom trace spans that nest under the automatic platform spans in the Workers Observability waterfall.
- `Span::set_attribute` / `Span::is_traced` attach metadata and check sampling.
- `with_active_span` exposes the innermost open span so a `tracing_subscriber::Layer` can forward `tracing` events/fields onto it.

The new `custom-spans` example shows a ready-made `WorkersLayer` doing exactly that. Addresses #899.
