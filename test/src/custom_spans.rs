use super::SomeSharedData;
use worker::observability::{
    active_span, enter_span, enter_span_async, start_active_span, start_span,
};
use worker::{js_sys, Delay, Env, Request, Response, Result};

/// Sync spans: the body runs exactly once, its value comes back out, and
/// attributes of every supported type can be set and chained.
pub fn handle_enter_span(_req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let mut calls = 0;
    let value = enter_span("outer", |span| {
        calls += 1;
        span.set_attribute("kind", "sync")
            .set_attribute("count", 7)
            .set_attribute("ratio", 0.5)
            .set_attribute("ok", true)
            .set_attribute("owned", String::from("string"));
        let inner = enter_span("inner", |child| {
            child.set_attribute("depth", 2u8);
            21
        });
        inner * 2
    });

    Response::from_json(&serde_json::json!({
        "value": value,
        "calls": calls,
    }))
}

/// Async spans: the body's future is driven to completion inside the span,
/// including across an `.await`, and its value comes back out.
#[worker::send]
pub async fn handle_enter_span_async(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let value = enter_span_async("async_outer", |span| async move {
        span.set_attribute("phase", "before");
        Delay::from(std::time::Duration::from_millis(5)).await;
        span.set_attribute("phase", "after");
        enter_span("after_await", |child| {
            child.set_attribute("nested", true);
            String::from("done")
        })
    })
    .await;

    Response::from_json(&serde_json::json!({ "value": value }))
}

/// Manually ended spans: `start_span` returns a handle, `end` is idempotent,
/// and calls after `end` are ignored rather than rejected.
///
/// `Span::span_context`, `Span::set_status`, and `Span::update_name` are not
/// exercised here: the workerd pinned by this repo's miniflare predates them.
pub fn handle_start_span(_req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let span = start_span("manual");
    span.set_attribute("step", 1);
    let traced_before = span.is_traced();
    span.end();
    span.end();
    span.set_attribute("step", 2);

    Response::from_json(&serde_json::json!({
        "traced_before_end": traced_before,
        "traced_after_end": span.is_traced(),
    }))
}

/// `start_active_span` hands out an owned handle that outlives the body, and
/// exceptions can be recorded from Rust errors and from JS errors.
pub fn handle_start_active_span(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let (child_value, span) = start_active_span("active", |span| {
        let child = enter_span("child_of_active", |_| 5);
        (child, span)
    });
    span.record_exception("plain message");
    span.record_exception(&worker::Error::RustError("rust error".into()));
    span.record_js_exception(&js_sys::Error::new("js error").into());
    span.end();

    Response::from_json(&serde_json::json!({ "child_value": child_value }))
}

/// `active_span` sees the invocation's root span, and inside a custom span it
/// still resolves to a span.
pub fn handle_active_span(_req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let root = active_span();
    let inside = enter_span("outer", |_span| active_span().is_some());

    Response::from_json(&serde_json::json!({
        "root_present": root.is_some(),
        "inside_present": inside,
    }))
}
