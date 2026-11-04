# Custom trace spans

Records Rust spans in the Workers Observability trace waterfall, next to the automatic `fetch`
spans:

- `enter_span_async` / `enter_span` wrap the request and its steps in spans.
- Two upstream fetches run concurrently, each nested under its own span.
- `WorkersLayer` (`src/layer.rs`) records `tracing` spans and events as platform spans and
  attributes, so `#[instrument]` and `info!` show up with no Workers-specific code.

Tracing must be enabled for the Worker; see `wrangler.toml`.

```sh
npx wrangler deploy
```
