//! Emscripten-on-Workers example.
//!
//! Demonstrates `std` library APIs and **tokio** on `wasm32-unknown-emscripten`.
//! These either panic or fail to compile on `wasm32-unknown-unknown`:
//!
//! - `std::time::Instant` / `SystemTime` (clock access)
//! - `std::env::current_dir` / `std::env::vars` (process environment)
//! - `std::collections::HashMap` with default random state
//! - `std::fs` write / read / metadata / remove (in-memory VFS via MEMFS)
//! - `rand::random` (WASI `random_get` import)
//! - `tokio::spawn`, `tokio::sync::mpsc`, `tokio::sync::oneshot`, `tokio::join!`

use std::collections::HashMap;
use std::fmt::Write;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use wasm_bindgen::prelude::*;

/// Async entry point — returns an HTML page showing test results.
#[wasm_bindgen]
pub async fn handle_request() -> String {
    let start = Instant::now();

    let checks = vec![
        check_system_time(),
        check_current_dir(),
        check_env_vars(),
        check_hashmap(),
        check_random(),
        check_filesystem(),
        check_tokio().await,
    ];

    let elapsed = start.elapsed();
    let all_passed = checks.iter().all(|c| c.passed);
    let pass_count = checks.iter().filter(|c| c.passed).count();
    let total = checks.len();

    let mut html = String::new();
    let _ = write!(
        html,
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Emscripten on Workers</title>
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{ font-family: system-ui, -apple-system, sans-serif; background: #0a0a0a; color: #e5e5e5; padding: 2rem; }}
  .container {{ max-width: 640px; margin: 0 auto; }}
  h1 {{ font-size: 1.5rem; font-weight: 600; margin-bottom: 0.25rem; }}
  .subtitle {{ color: #888; font-size: 0.875rem; margin-bottom: 1.5rem; }}
  .summary {{ display: flex; align-items: center; gap: 0.75rem; padding: 1rem; border-radius: 0.5rem; margin-bottom: 1.5rem; background: {}; }}
  .summary-icon {{ font-size: 1.25rem; }}
  .summary-text {{ font-weight: 500; }}
  .checks {{ display: flex; flex-direction: column; gap: 0.5rem; }}
  .check {{ display: flex; align-items: flex-start; gap: 0.75rem; padding: 0.75rem 1rem; border-radius: 0.5rem; background: #141414; border: 1px solid #222; }}
  .check-icon {{ flex-shrink: 0; width: 1.25rem; text-align: center; }}
  .check-pass {{ color: #22c55e; }}
  .check-fail {{ color: #ef4444; }}
  .check-name {{ font-weight: 500; font-size: 0.875rem; }}
  .check-detail {{ color: #888; font-size: 0.8125rem; margin-top: 0.125rem; font-family: ui-monospace, monospace; }}
  .footer {{ margin-top: 1.5rem; color: #555; font-size: 0.75rem; text-align: center; }}
</style>
</head>
<body>
<div class="container">
<h1>Emscripten on Workers</h1>
<p class="subtitle">Rust std library APIs running on wasm32-unknown-emscripten</p>
<div class="summary">
  <span class="summary-icon">{}</span>
  <span class="summary-text">{pass_count}/{total} checks passed</span>
</div>
<div class="checks">"#,
        if all_passed { "#0a1f0a" } else { "#1f0a0a" },
        if all_passed { "&#10003;" } else { "&#10007;" },
    );

    for check in &checks {
        let (icon_class, icon) = if check.passed {
            ("check-pass", "&#10003;")
        } else {
            ("check-fail", "&#10007;")
        };
        let _ = write!(
            html,
            r#"
<div class="check">
  <span class="check-icon {icon_class}">{icon}</span>
  <div>
    <div class="check-name">{}</div>
    <div class="check-detail">{}</div>
  </div>
</div>"#,
            check.name,
            html_escape(&check.detail),
        );
    }

    let _ = write!(
        html,
        r#"
</div>
<p class="footer">Completed in {} &micro;s</p>
</div>
</body>
</html>"#,
        elapsed.as_micros(),
    );

    html
}

struct Check {
    name: &'static str,
    detail: String,
    passed: bool,
}

fn check_system_time() -> Check {
    let epoch_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    Check {
        name: "SystemTime",
        detail: format!("epoch_ms = {epoch_ms}"),
        passed: epoch_ms > 1_700_000_000_000,
    }
}

fn check_current_dir() -> Check {
    match std::env::current_dir() {
        Ok(p) => Check {
            name: "Current directory",
            detail: format!("{}", p.display()),
            passed: true,
        },
        Err(e) => Check {
            name: "Current directory",
            detail: format!("error: {e}"),
            passed: false,
        },
    }
}

fn check_env_vars() -> Check {
    let count = std::env::vars().count();
    Check {
        name: "Environment variables",
        detail: format!("{count} vars"),
        passed: count > 0,
    }
}

fn check_hashmap() -> Check {
    let mut map = HashMap::new();
    map.insert("hello", 1);
    map.insert("from", 2);
    map.insert("emscripten", 3);
    let sum: i32 = map.values().sum();
    Check {
        name: "HashMap (random state)",
        detail: format!("sum of 3 entries = {sum}"),
        passed: sum == 6,
    }
}

fn check_random() -> Check {
    let val: u64 = rand::random();
    Check {
        name: "rand::random()",
        detail: format!("{val}"),
        passed: true,
    }
}

fn check_filesystem() -> Check {
    let path = "/tmp/emscripten_demo.txt";
    let payload = "Emscripten MEMFS works!";

    let result = (|| -> Result<usize, String> {
        std::fs::write(path, payload).map_err(|e| format!("write: {e}"))?;
        let read_back = std::fs::read_to_string(path).map_err(|e| format!("read: {e}"))?;
        if read_back != payload {
            return Err(format!("mismatch: expected {payload:?}, got {read_back:?}"));
        }
        let len = std::fs::metadata(path)
            .map_err(|e| format!("metadata: {e}"))?
            .len() as usize;
        std::fs::remove_file(path).map_err(|e| format!("remove: {e}"))?;
        Ok(len)
    })();

    match result {
        Ok(len) => Check {
            name: "Filesystem (MEMFS)",
            detail: format!("write/read/stat/remove ok, {len} bytes"),
            passed: true,
        },
        Err(e) => Check {
            name: "Filesystem (MEMFS)",
            detail: e,
            passed: false,
        },
    }
}

async fn check_tokio() -> Check {
    use tokio::sync::{mpsc, oneshot};

    // mpsc channel: fan-in from spawned tasks
    let (tx, mut rx) = mpsc::channel::<i32>(8);
    for i in 1..=3 {
        let tx = tx.clone();
        wasm_bindgen_futures::spawn_local(async move {
            tx.send(i * 10).await.ok();
        });
    }
    drop(tx);

    let mut mpsc_sum = 0;
    while let Some(val) = rx.recv().await {
        mpsc_sum += val;
    }

    // oneshot: request/reply
    let (os_tx, os_rx) = oneshot::channel::<&str>();
    wasm_bindgen_futures::spawn_local(async move {
        os_tx.send("pong").ok();
    });
    let oneshot_reply = os_rx.await.unwrap_or("error");

    // join!: concurrent futures
    let (a, b) = tokio::join!(async { 100 }, async { 200 });
    let join_sum = a + b;

    let passed = mpsc_sum == 60 && oneshot_reply == "pong" && join_sum == 300;
    Check {
        name: "Tokio (spawn, mpsc, oneshot, join!)",
        detail: format!("mpsc_sum={mpsc_sum} oneshot={oneshot_reply} join_sum={join_sum}"),
        passed,
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
