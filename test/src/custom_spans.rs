use super::SomeSharedData;
use std::time::Duration;
use worker::observability::{
    active_span, enter_span, enter_span_async, start_span, Span, SpanStatus,
};
use worker::{js_sys, Delay, Env, Request, Response, Result};

fn is_active(span: &Span) -> bool {
    active_span().as_ref() == Some(span)
}

/// Sync spans: values come back out, nesting follows `enter_span` / `in_scope`, the enclosing
/// span is restored afterwards, and the span API is callable on open and ended spans.
pub fn handle_spans(_req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let root = active_span().expect("inside an invocation");
    let mut calls = 0;

    let (value, outer_active, inner_active) = enter_span("outer", |outer| {
        calls += 1;
        outer
            .set_attribute("kind", "sync")
            .set_attribute("count", 7)
            .set_attribute("ratio", 0.5)
            .set_attribute("ok", true)
            .set_attribute("owned", String::from("string"));
        let (inner, inner_active) = enter_span("inner", |inner| (21, is_active(inner)));
        (inner * 2, is_active(outer), inner_active)
    });

    let manual = start_span("manual");
    let manual_active_before = is_active(&manual);
    let manual_active_in_scope = manual.in_scope(|| is_active(&manual));
    manual.record_exception("plain message");
    manual.record_exception(&worker::Error::RustError("rust error".into()));
    manual.record_js_exception(&js_sys::Error::new("js error").into());
    manual
        .update_name("manual-renamed")
        .set_status(SpanStatus::Error(Some("failed".into())));
    let _ = manual.span_context();
    manual.end();
    manual.end();
    manual.set_attribute("after_end", true);

    Response::from_json(&serde_json::json!({
        "value": value,
        "calls": calls,
        "outer_active": outer_active,
        "inner_active": inner_active,
        "manual_active_before": manual_active_before,
        "manual_active_in_scope": manual_active_in_scope,
        "root_restored": is_active(&root),
    }))
}

/// Async spans stay active across `.await`s, and borrowed (non-`'static`) state can be used
/// inside them.
#[worker::send]
pub async fn handle_async_spans(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let root = active_span().expect("inside an invocation");
    let owned = String::from("borrowed");
    let borrowed = &owned;

    let (before, after, nested, value) = enter_span_async("async-outer", |span| async move {
        let before = is_active(&span);
        Delay::from(Duration::from_millis(5)).await;
        let after = is_active(&span);
        let nested = enter_span_async("async-inner", |inner| async move {
            Delay::from(Duration::from_millis(1)).await;
            is_active(&inner)
        })
        .await;
        (before, after && is_active(&span), nested, borrowed.len())
    })
    .await;

    Response::from_json(&serde_json::json!({
        "before": before,
        "after": after,
        "nested": nested,
        "value": value,
        "root_restored": is_active(&root),
    }))
}

/// Concurrent spans polled by the same executor tick each see their own span on every poll.
#[worker::send]
pub async fn handle_concurrent_spans(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let task = |name: &'static str, ms: u64| {
        enter_span_async(name, move |span| async move {
            let mut seen = vec![is_active(&span)];
            for _ in 0..3 {
                Delay::from(Duration::from_millis(ms)).await;
                seen.push(is_active(&span));
            }
            seen
        })
    };
    let (a, b) = futures_util::future::join(task("task-a", 3), task("task-b", 7)).await;

    Response::from_json(&serde_json::json!({ "a": a, "b": b }))
}
