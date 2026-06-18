# custom-spans

Custom trace spans for Workers Observability, in Rust — using
[`worker::observability`](../../worker/src/observability.rs).

It demonstrates:

- `enter_span_async("handle_request", |span| async move { … })` — an async
  root span around the request handler.
- `enter_span("load_rows", |span| …)` — a nested sync span that auto-parents
  under the root via the JS async context.
- `span.set_attribute(...)` and `span.is_traced()`.
- `WorkersLayer` (the `tracing` feature) forwarding ordinary `tracing::info!`
  events onto the active platform span as attributes.

Custom spans are recorded only when tracing is enabled in your Worker's
observability config — see `wrangler.toml` (`[observability.traces]`).

```sh
npx wrangler deploy
```

Then open the Worker's **Observability → Traces** view and trigger a request;
`handle_request` and its nested `load_rows` span appear in the waterfall next to
the automatic `fetch` span.
