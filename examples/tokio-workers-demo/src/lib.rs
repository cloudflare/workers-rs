//! # Tokio-Workers Demo
//!
//! This example demonstrates how `tokio-workers` provides a drop-in replacement
//! for common tokio APIs in Cloudflare Workers.
//!
//! ## The Problem
//!
//! Standard tokio code like this won't work on Workers:
//!
//! ```rust,ignore
//! use tokio::{spawn, time::sleep, sync::mpsc};
//!
//! #[tokio::main]  // <-- tokio runtime can't run in WASM
//! async fn main() {
//!     let handle = spawn(async { 42 });  // <-- needs tokio runtime
//!     sleep(Duration::from_secs(1)).await;  // <-- needs tokio timer
//!     // ...
//! }
//! ```
//!
//! ## The Solution
//!
//! Replace `use tokio::*` with `use tokio_workers::*`:
//!
//! ```rust,ignore
//! use tokio_workers::{spawn, time::sleep, sync::mpsc};
//! // Now the same code works on Workers!
//! ```

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use worker::*;

// =============================================================================
// THE KEY CHANGE: Instead of importing from tokio, import from tokio_workers
// =============================================================================
//
// BEFORE (standard tokio - won't work on Workers):
//   use tokio::{spawn, time::{sleep, timeout}, sync::{mpsc, Mutex}, join};
//
// AFTER (tokio-workers - works on Workers!):
use tokio_workers::{
    join, spawn,
    sync::{mpsc, Mutex},
    task::spawn_blocking,
    time::{sleep, timeout},
};

#[derive(Debug, Serialize, Deserialize)]
struct DemoResponse {
    endpoint: String,
    description: String,
    result: String,
    elapsed_ms: f64,
}

fn now() -> f64 {
    js_sys::Date::now()
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get("/", index)
        .get_async("/spawn", demo_spawn)
        .get_async("/sleep", demo_sleep)
        .get_async("/timeout", demo_timeout)
        .get_async("/channels", demo_channels)
        .get_async("/mutex", demo_mutex)
        .get_async("/concurrent", demo_concurrent)
        .get_async("/spawn-blocking", demo_spawn_blocking)
        .run(req, env)
        .await
}

fn index(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    Response::ok(
        r#"Tokio-Workers Demo

This Worker demonstrates tokio-workers as a drop-in replacement for tokio.

Endpoints:
  GET /spawn          - Spawn async tasks and await results
  GET /sleep?ms=100   - Sleep for specified milliseconds  
  GET /timeout        - Demonstrate timeout behavior
  GET /channels       - Use mpsc channels for communication
  GET /mutex          - Use async Mutex for shared state
  GET /concurrent     - Run multiple operations with join!
  GET /spawn-blocking - Run blocking code (sync mode)

All endpoints return JSON with timing information.
"#,
    )
}

/// Demonstrates: tokio::spawn -> tokio_workers::spawn
///
/// In standard tokio:
/// ```rust,ignore
/// let handle = tokio::spawn(async { compute() });
/// let result = handle.await?;
/// ```
async fn demo_spawn(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let start = now();

    // Spawn multiple concurrent tasks
    let h1 = spawn(async {
        sleep(Duration::from_millis(10)).await;
        1 + 1
    });
    let h2 = spawn(async {
        sleep(Duration::from_millis(10)).await;
        2 * 2
    });
    let h3 = spawn(async {
        sleep(Duration::from_millis(10)).await;
        3 + 3
    });

    // Await all handles
    let r1 = h1.await.map_err(|e| Error::RustError(e.to_string()))?;
    let r2 = h2.await.map_err(|e| Error::RustError(e.to_string()))?;
    let r3 = h3.await.map_err(|e| Error::RustError(e.to_string()))?;

    let elapsed = now() - start;

    Response::from_json(&DemoResponse {
        endpoint: "/spawn".into(),
        description: "Spawned 3 tasks with 10ms sleep each, ran concurrently".into(),
        result: format!("Results: {}, {}, {} (sum={})", r1, r2, r3, r1 + r2 + r3),
        elapsed_ms: elapsed,
    })
}

/// Demonstrates: tokio::time::sleep -> tokio_workers::time::sleep
///
/// In standard tokio:
/// ```rust,ignore
/// tokio::time::sleep(Duration::from_millis(100)).await;
/// ```
async fn demo_sleep(req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let ms: u64 = req
        .url()?
        .query_pairs()
        .find(|(k, _)| k == "ms")
        .map(|(_, v)| v.parse().unwrap_or(100))
        .unwrap_or(100);

    let start = now();
    sleep(Duration::from_millis(ms)).await;
    let elapsed = now() - start;

    Response::from_json(&DemoResponse {
        endpoint: "/sleep".into(),
        description: format!("Slept for {}ms using tokio_workers::time::sleep", ms),
        result: format!("Requested: {}ms, Actual: {:.1}ms", ms, elapsed),
        elapsed_ms: elapsed,
    })
}

/// Demonstrates: tokio::time::timeout -> tokio_workers::time::timeout
///
/// In standard tokio:
/// ```rust,ignore
/// match tokio::time::timeout(Duration::from_millis(50), slow_operation()).await {
///     Ok(result) => println!("Completed: {}", result),
///     Err(_) => println!("Timed out!"),
/// }
/// ```
async fn demo_timeout(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let start = now();

    // This should complete (50ms timeout, 10ms operation)
    let fast_result = timeout(Duration::from_millis(50), async {
        sleep(Duration::from_millis(10)).await;
        "fast operation completed"
    })
    .await;

    // This should timeout (20ms timeout, 100ms operation)
    let slow_result = timeout(Duration::from_millis(20), async {
        sleep(Duration::from_millis(100)).await;
        "slow operation completed"
    })
    .await;

    let elapsed = now() - start;

    let result = format!(
        "Fast (50ms timeout, 10ms op): {:?}\nSlow (20ms timeout, 100ms op): {:?}",
        fast_result
            .map(|s| s.to_string())
            .map_err(|e| e.to_string()),
        slow_result
            .map(|s| s.to_string())
            .map_err(|e| e.to_string()),
    );

    Response::from_json(&DemoResponse {
        endpoint: "/timeout".into(),
        description: "Demonstrated timeout with fast (succeeds) and slow (times out) operations"
            .into(),
        result,
        elapsed_ms: elapsed,
    })
}

/// Demonstrates: tokio::sync::mpsc -> tokio_workers::sync::mpsc
///
/// In standard tokio:
/// ```rust,ignore
/// let (tx, mut rx) = tokio::sync::mpsc::channel(32);
/// tokio::spawn(async move { tx.send(42).await });
/// let value = rx.recv().await;
/// ```
async fn demo_channels(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let start = now();

    let (tx, mut rx) = mpsc::channel::<i32>(10);

    // Spawn a producer task
    let tx1 = tx.clone();
    spawn(async move {
        for i in 1..=5 {
            sleep(Duration::from_millis(5)).await;
            let _ = tx1.send(i).await;
        }
    });

    // Spawn another producer
    let tx2 = tx.clone();
    spawn(async move {
        for i in 6..=10 {
            sleep(Duration::from_millis(5)).await;
            let _ = tx2.send(i).await;
        }
    });

    // Drop original tx so channel closes when producers finish
    drop(tx);

    // Collect all values
    let mut values = Vec::new();
    while let Some(v) = rx.recv().await {
        values.push(v);
    }

    let elapsed = now() - start;

    Response::from_json(&DemoResponse {
        endpoint: "/channels".into(),
        description: "Two producers sent values 1-5 and 6-10 through mpsc channel".into(),
        result: format!("Received {} values: {:?}", values.len(), values),
        elapsed_ms: elapsed,
    })
}

/// Demonstrates: tokio::sync::Mutex -> tokio_workers::sync::Mutex
///
/// In standard tokio:
/// ```rust,ignore
/// let counter = Arc::new(tokio::sync::Mutex::new(0));
/// let c = counter.clone();
/// tokio::spawn(async move { *c.lock().await += 1; });
/// ```
async fn demo_mutex(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let start = now();

    let counter = Arc::new(Mutex::new(0));

    // Spawn multiple tasks that increment the counter
    let mut handles = Vec::new();
    for _ in 0..10 {
        let c = counter.clone();
        handles.push(spawn(async move {
            let mut lock = c.lock().await;
            *lock += 1;
        }));
    }

    // Wait for all tasks to complete
    for h in handles {
        h.await.map_err(|e| Error::RustError(e.to_string()))?;
    }

    let final_value = *counter.lock().await;
    let elapsed = now() - start;

    Response::from_json(&DemoResponse {
        endpoint: "/mutex".into(),
        description: "10 concurrent tasks each incremented a shared counter".into(),
        result: format!("Final counter value: {} (expected: 10)", final_value),
        elapsed_ms: elapsed,
    })
}

/// Demonstrates: tokio::join! -> tokio_workers::join!
///
/// In standard tokio:
/// ```rust,ignore
/// let (a, b, c) = tokio::join!(
///     async { 1 },
///     async { 2 },
///     async { 3 },
/// );
/// ```
async fn demo_concurrent(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let start = now();

    // Run multiple async operations concurrently
    let (a, b, c) = join!(
        async {
            sleep(Duration::from_millis(30)).await;
            "operation A"
        },
        async {
            sleep(Duration::from_millis(30)).await;
            "operation B"
        },
        async {
            sleep(Duration::from_millis(30)).await;
            "operation C"
        },
    );

    let elapsed = now() - start;

    Response::from_json(&DemoResponse {
        endpoint: "/concurrent".into(),
        description: "Three 30ms operations ran concurrently with join!".into(),
        result: format!("Results: {}, {}, {} (should take ~30ms, not 90ms)", a, b, c),
        elapsed_ms: elapsed,
    })
}

/// Demonstrates: tokio::task::spawn_blocking -> tokio_workers::task::spawn_blocking
///
/// Note: In WASM, true blocking isn't possible. With `spawn-blocking-sync` feature,
/// the closure runs synchronously (blocking the event loop).
///
/// In standard tokio:
/// ```rust,ignore
/// let result = tokio::task::spawn_blocking(|| {
///     expensive_computation()
/// }).await?;
/// ```
async fn demo_spawn_blocking(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let start = now();

    // This runs synchronously in WASM (with spawn-blocking-sync feature)
    let handle = spawn_blocking(|| {
        // Simulate some "blocking" computation
        let mut sum = 0u64;
        for i in 0..1000 {
            sum += i;
        }
        sum
    });

    let result = handle.await.map_err(|e| Error::RustError(e.to_string()))?;
    let elapsed = now() - start;

    Response::from_json(&DemoResponse {
        endpoint: "/spawn-blocking".into(),
        description: "Ran 'blocking' computation (sync in WASM with spawn-blocking-sync feature)"
            .into(),
        result: format!("Sum of 0..1000 = {}", result),
        elapsed_ms: elapsed,
    })
}
