//! Custom trace spans for Workers Observability, in Rust.
//!
//! Wraps the request in an async platform span, nests a sync span under it,
//! and lets the `WorkersLayer` forward `tracing` events onto the active span.
//! Deploy with `observability.traces` enabled (see `wrangler.toml`) and the
//! spans appear in the trace waterfall alongside the automatic `fetch` span.

mod layer;

use layer::WorkersLayer;
use tracing::info;
use tracing_subscriber::prelude::*;
use worker::observability::{enter_span, enter_span_async};
use worker::{event, Context, Env, Request, Response, Result};

#[event(start)]
fn start() {
    console_error_panic_hook::set_once();
    // `try_init` so a hot-reloaded isolate doesn't panic on a second install.
    let _ = tracing_subscriber::registry().with(WorkersLayer).try_init();
}

#[event(fetch)]
async fn fetch(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let path = req.path();

    enter_span_async("handle_request", move |span| async move {
        span.set_attribute("http.path", path.as_str());
        span.set_attribute("sampled", span.is_traced());

        // A plain `tracing` event — the layer forwards `user_id` as an
        // attribute on `handle_request`. No platform-specific code here.
        info!(user_id = 42, "request received");

        // A nested sync span; auto-parents under `handle_request`.
        let rows = enter_span("load_rows", |child| {
            let rows = expensive_query();
            child.set_attribute("db.rows", rows);
            rows
        });

        info!(rows, "query complete");
        Response::ok(format!("loaded {rows} rows for {path}"))
    })
    .await
}

/// Stand-in for real work — a Worker would hit D1 / KV / a binding here.
fn expensive_query() -> u32 {
    1234
}
