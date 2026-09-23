//! Stock Tokio on Workers. With the `worker` crate's `tokio` feature every
//! `#[event]` handler runs on its own Tokio event loop, whose wait is the
//! Workers event loop, so `tokio::spawn`, timers and sync primitives work as
//! they do natively: no `#[tokio::main]`, no runtime setup.

use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{sleep, timeout};
use worker::*;

fn main() {}

#[derive(Serialize)]
struct Demo {
    description: &'static str,
    result: String,
    elapsed_ms: f64,
}

fn now() -> f64 {
    js_sys::Date::now()
}

fn json(description: &'static str, result: String, start: f64) -> Result<Response> {
    Response::from_json(&Demo {
        description,
        result,
        elapsed_ms: now() - start,
    })
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get("/", |_, _| {
            Response::ok(
                "Tokio on Workers\n\n\
                 GET /spawn      spawn tasks and await their JoinHandles\n\
                 GET /sleep?ms=  tokio::time::sleep\n\
                 GET /timeout    tokio::time::timeout\n\
                 GET /channels   mpsc between spawned producers and the handler\n\
                 GET /mutex      tokio::sync::Mutex shared across tasks\n\
                 GET /join       tokio::join! over concurrent futures\n",
            )
        })
        .get_async("/spawn", spawn)
        .get_async("/sleep", sleep_for)
        .get_async("/timeout", timeouts)
        .get_async("/channels", channels)
        .get_async("/mutex", mutex)
        .get_async("/join", join)
        .run(req, env)
        .await
}

async fn spawn(_: Request, _: RouteContext<()>) -> Result<Response> {
    let start = now();
    let handles: Vec<_> = (1..=3)
        .map(|i| {
            tokio::spawn(async move {
                sleep(Duration::from_millis(10)).await;
                i * i
            })
        })
        .collect();
    let mut results = Vec::new();
    for h in handles {
        results.push(h.await.map_err(|e| Error::RustError(e.to_string()))?);
    }
    json(
        "3 spawned tasks, each sleeping 10ms, run concurrently",
        format!("{results:?}"),
        start,
    )
}

async fn sleep_for(req: Request, _: RouteContext<()>) -> Result<Response> {
    let ms: u64 = req
        .url()?
        .query_pairs()
        .find(|(k, _)| k == "ms")
        .and_then(|(_, v)| v.parse().ok())
        .unwrap_or(100);
    let start = now();
    sleep(Duration::from_millis(ms)).await;
    json("tokio::time::sleep", format!("slept {ms}ms"), start)
}

async fn timeouts(_: Request, _: RouteContext<()>) -> Result<Response> {
    let start = now();
    let fast = timeout(Duration::from_millis(50), sleep(Duration::from_millis(10))).await;
    let slow = timeout(Duration::from_millis(20), sleep(Duration::from_millis(100))).await;
    json(
        "a 10ms operation under a 50ms timeout, then a 100ms one under 20ms",
        format!("fast: {}, slow: {}", verdict(fast), verdict(slow)),
        start,
    )
}

fn verdict(r: std::result::Result<(), tokio::time::error::Elapsed>) -> &'static str {
    match r {
        Ok(()) => "completed",
        Err(_) => "timed out",
    }
}

async fn channels(_: Request, _: RouteContext<()>) -> Result<Response> {
    let start = now();
    let (tx, mut rx) = mpsc::channel::<u32>(4);
    for range in [1..=5, 6..=10] {
        let tx = tx.clone();
        tokio::spawn(async move {
            for i in range {
                sleep(Duration::from_millis(5)).await;
                let _ = tx.send(i).await;
            }
        });
    }
    drop(tx);
    let mut received = Vec::new();
    while let Some(v) = rx.recv().await {
        received.push(v);
    }
    json(
        "two producers send 1..=5 and 6..=10 over an mpsc channel",
        format!("{received:?}"),
        start,
    )
}

async fn mutex(_: Request, _: RouteContext<()>) -> Result<Response> {
    let start = now();
    let counter = Arc::new(Mutex::new(0));
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let counter = counter.clone();
            tokio::spawn(async move {
                let mut n = counter.lock().await;
                sleep(Duration::from_millis(1)).await;
                *n += 1;
            })
        })
        .collect();
    for h in handles {
        h.await.map_err(|e| Error::RustError(e.to_string()))?;
    }
    let n = *counter.lock().await;
    json(
        "10 tasks increment a counter while holding an async Mutex",
        format!("counter = {n}"),
        start,
    )
}

async fn join(_: Request, _: RouteContext<()>) -> Result<Response> {
    let start = now();
    let (a, b, c) = tokio::join!(
        async {
            sleep(Duration::from_millis(30)).await;
            "a"
        },
        async {
            sleep(Duration::from_millis(30)).await;
            "b"
        },
        async {
            sleep(Duration::from_millis(30)).await;
            "c"
        },
    );
    json(
        "three 30ms futures joined; takes ~30ms, not 90",
        format!("{a}{b}{c}"),
        start,
    )
}
