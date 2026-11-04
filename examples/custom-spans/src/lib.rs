//! Custom trace spans in Rust. Deploy with `observability.traces` enabled (see `wrangler.toml`)
//! and the spans appear in the trace waterfall alongside the automatic `fetch` span.

mod layer;

use futures_util::future::join;
use layer::WorkersLayer;
use tracing::{info, instrument};
use tracing_subscriber::prelude::*;
use worker::observability::{active_span, enter_span, enter_span_async, SpanStatus};
use worker::{event, Context, Env, Error, Fetch, Request, Response, Result};

#[event(start)]
fn start() {
    let _ = tracing_subscriber::registry().with(WorkersLayer).try_init();
}

#[event(fetch)]
async fn fetch(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let path = req.path();

    // Rename the invocation's root span once the route is known.
    if let Some(root) = active_span() {
        root.update_name(format!("{} {path}", req.method()));
    }

    enter_span_async("handle_request", |span| async move {
        span.set_attribute("http.path", path.as_str());
        // Recorded by the layer as an attribute on `handle_request`.
        info!(user_id = 42, "request received");

        // Two upstream fetches run concurrently, each nested under its own span.
        let (a, b) = join(
            enter_span_async("upstream_a", |_| fetch_len("https://example.com/")),
            enter_span_async("upstream_b", |_| fetch_len("https://example.org/")),
        )
        .await;
        let (a, b) = match (a, b) {
            (Ok(a), Ok(b)) => (a, b),
            (Err(err), _) | (_, Err(err)) => {
                span.record_exception(&err);
                span.set_status(SpanStatus::Error(Some("upstream failed".into())));
                return Response::error(err.to_string(), 502);
            }
        };

        let total = enter_span("summarize", |child| {
            child.set_attribute("bytes", a + b);
            summarize(a, b)
        });

        span.set_status(SpanStatus::Ok);
        let mut resp = Response::ok(format!("{total} bytes"))?;
        resp.headers_mut()
            .set("x-trace-id", &span.span_context().trace_id)?;
        Ok(resp)
    })
    .await
}

async fn fetch_len(url: &str) -> Result<usize> {
    let mut resp = Fetch::Url(url.parse().map_err(|e| Error::RustError(format!("{e}")))?)
        .send()
        .await?;
    Ok(resp.bytes().await?.len())
}

/// A plain `tracing` span: the layer records it as a child of `summarize`.
#[instrument]
fn summarize(a: usize, b: usize) -> usize {
    info!("summarizing");
    a + b
}
