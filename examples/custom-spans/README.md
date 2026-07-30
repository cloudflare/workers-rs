# custom-spans

Custom trace spans for Workers Observability, in Rust — using
[`worker::observability`](../../worker/src/observability.rs).

It demonstrates:

- `enter_span_async("handle_request", |span| async move { … })` — an async
  root span around the request handler.
- `enter_span("load_rows", |span| …)` — a nested sync span that auto-parents
  under the root via the JS async context.
- `span.set_attribute(...)` and `span.is_traced()`.
- `WorkersLayer` — a `tracing_subscriber::Layer` bridging `tracing` onto the
  platform: every `span!` / `#[instrument]` becomes a platform span for its
  full lifetime (via `start_active_span` + `Span::end`), and `tracing::info!`
  events land as attributes on the span they were emitted in. `summarize()` in
  `src/lib.rs` is instrumented that way, with no Workers-specific code.

Nesting note: a bridged span parents under the nearest enclosing `enter_span`,
not under its `tracing` parent, because the platform derives hierarchy from the
JS async context. Wrap a subtree in `enter_span` where the shape matters.

Custom spans are recorded only when tracing is enabled in your Worker's
observability config — see `wrangler.toml` (`[observability.traces]`).

```sh
npx wrangler deploy
```

Then open the Worker's **Observability → Traces** view and trigger a request;
`handle_request`, its nested `load_rows` span, and the `tracing`-instrumented
`summarize` span appear in the waterfall next to the automatic `fetch` span.
