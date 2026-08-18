//! Experiment worker for the wasm `memory.discard` instruction.
//!
//! Built with jemalloc as the global allocator (dirty_decay_ms:0), so freed
//! pages are immediately purged via `madvise(MADV_DONTNEED)`, which lowers to
//! a single `memory.discard` instruction. The `/churn` endpoint allocates,
//! touches and frees memory so resident set can be observed from outside.

use worker::*;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

const WASM_PAGE: usize = 65536;
const HOST_PAGE: usize = 4096;

fn memory_bytes() -> usize {
    core::arch::wasm32::memory_size(0) * WASM_PAGE
}

fn mb(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

/// Allocate `total_mb` in `chunk_kb` chunks, touching every host page, then
/// free everything. Returns peak/allocated stats.
fn churn(total_mb: usize, chunk_kb: usize) -> String {
    let chunk_bytes = chunk_kb * 1024;
    let chunks = (total_mb * 1024 * 1024) / chunk_bytes;

    let mut held: Vec<Vec<u8>> = Vec::with_capacity(chunks);
    for i in 0..chunks {
        let mut buf = vec![0u8; chunk_bytes];
        // Touch every host page to make it resident.
        let mut off = 0;
        while off < chunk_bytes {
            buf[off] = (i & 0xff) as u8;
            off += HOST_PAGE;
        }
        held.push(buf);
    }
    let peak = memory_bytes();
    // Prevent the writes from being optimized out.
    let sum: usize = held.iter().map(|b| b[0] as usize).sum();
    drop(held);
    let after = memory_bytes();

    format!(
        r#"{{"allocated_mb":{},"chunk_kb":{},"linear_peak_mb":{},"linear_after_mb":{},"check":{}}}"#,
        total_mb,
        chunk_kb,
        mb(peak),
        mb(after),
        sum
    )
}

#[event(fetch)]
async fn fetch(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let url = req.url()?;
    let query: std::collections::HashMap<String, String> = url
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let body = match url.path() {
        "/mem" => format!(r#"{{"linear_mb":{}}}"#, mb(memory_bytes())),
        "/churn" => {
            let total_mb: usize = query.get("mb").and_then(|v| v.parse().ok()).unwrap_or(64);
            let chunk_kb: usize = query
                .get("chunk_kb")
                .and_then(|v| v.parse().ok())
                .unwrap_or(64);
            churn(total_mb.min(96), chunk_kb.clamp(4, 4096))
        }
        _ => return Response::error("not found", 404),
    };

    Response::ok(body)
        .map(|resp| resp.with_headers(Headers::from_iter([("content-type", "application/json")])))
}
