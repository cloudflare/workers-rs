//! Custom trace spans for Workers Observability, in Rust.
//!
//! Wraps the request in an async platform span, nests a sync span under it,
//! and lets `WorkersLayer` turn ordinary `tracing` spans and events into
//! platform spans and attributes. Deploy with `observability.traces` enabled
//! (see `wrangler.toml`) and they appear in the trace waterfall alongside the
//! automatic `fetch` span.

mod layer;

use layer::WorkersLayer;
use tracing::{info, instrument};
use tracing_subscriber::prelude::*;
use worker::observability::{active_span, enter_span, enter_span_async, SpanStatus};
use worker::{event, Context, Env, Error, Request, Response, Result};

#[event(start)]
fn start() {
    console_error_panic_hook::set_once();
    // `try_init` so a hot-reloaded isolate doesn't panic on a second install.
    let _ = tracing_subscriber::registry().with(WorkersLayer).try_init();
}

#[event(fetch)]
async fn fetch(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let path = req.path();

    // The invocation's root span is active before any custom span is opened;
    // rename it after the route is known so traces group by route.
    if let Some(root) = active_span() {
        root.update_name(format!("{} {path}", req.method()).as_str());
    }

    enter_span_async("handle_request", move |span| async move {
        span.set_attribute("http.path", path.as_str())
            .set_attribute("sampled", span.is_traced());

        // A plain `tracing` event: the layer forwards `user_id` as an
        // attribute on `handle_request`. No platform-specific code here.
        info!(user_id = 42, "request received");

        if path == "/fail" {
            let err = Error::RustError("upstream returned an empty body".into());
            span.record_exception(&err);
            span.set_status(SpanStatus::error("lookup failed"));
            return Response::error(err.to_string(), 502);
        }

        // A nested sync span; auto-parents under `handle_request`.
        let rows = enter_span("load_rows", |child| {
            let rows = expensive_query();
            child.set_attribute("db.rows", rows);
            rows
        });

        // ...and the same work instrumented the ordinary `tracing` way. The
        // layer opens a platform span on entry and ends it on close, so this
        // gets a platform-measured duration with no Workers-specific code.
        let total = summarize(rows);

        info!(rows, "query complete");
        span.set_status(SpanStatus::Ok);

        // Hand the trace id to the client so a report can be matched to its
        // trace in the dashboard.
        let trace_id = span.span_context().trace_id;
        let mut resp = Response::ok(format!("loaded {rows} rows for {path}, total {total}"))?;
        resp.headers_mut().set("x-trace-id", &trace_id)?;
        Ok(resp)
    })
    .await
}

/// Stand-in for real work; a Worker would hit D1 / KV / a binding here.
fn expensive_query() -> u32 {
    1234
}

/// Instrumented with plain `tracing`: `WorkersLayer` bridges the span
/// lifetime onto `startSpan` / `span.end()`, and `rows` becomes an attribute.
#[instrument]
fn summarize(rows: u32) -> u32 {
    info!("summarizing");
    rows * 2
}
