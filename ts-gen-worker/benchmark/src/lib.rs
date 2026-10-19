#![cfg(any(js_sys_unstable_apis, target_arch = "wasm32"))]

use serde::{Deserialize, Serialize};
use ts_gen_worker::*;

#[allow(clippy::all)]
mod env;
use env::Env;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: ExecutionContext) -> Result<Response> {
    match req.path().as_str() {
        "/hello" => Response::builder().ok("hello"),
        "/headers" => handle_headers(&req),
        "/echo" => Response::builder().ok(req.text().await?),
        "/json" => handle_json(&req).await,
        "/kv" => handle_kv(&env).await,
        "/stream" => handle_stream(),
        "/fanout" => handle_fanout(&env).await,
        _ => Response::builder().error("Not Found", 404),
    }
}

fn handle_headers(req: &Request) -> Result<Response> {
    let headers = req.headers();
    let mut values = Vec::with_capacity(5);
    for name in ["x-h1", "x-h2", "x-h3", "x-h4", "x-h5"] {
        values.push(headers.get(name)?.unwrap_or_default());
    }
    Response::builder()
        .header("x-o1", &values[0])?
        .header("x-o2", &values[1])?
        .header("x-o3", &values[2])?
        .header("x-o4", &values[3])?
        .header("x-o5", &values[4])?
        .empty()
}

#[derive(Deserialize, Serialize)]
struct Payload {
    id: u64,
    name: String,
    active: bool,
    score: f64,
    tags: Vec<String>,
    items: Vec<Item>,
}

#[derive(Deserialize, Serialize)]
struct Item {
    id: u32,
    label: String,
    qty: u32,
    price: f64,
}

async fn handle_json(req: &Request) -> Result<Response> {
    let mut payload: Payload = req.json_into().await?;
    payload.id += 1;
    Response::builder().from_json(&payload)
}

async fn handle_kv(env: &Env) -> Result<Response> {
    let kv = env.kv();
    let value = match kv.get("bench").await? {
        Some(value) => value,
        None => "v".repeat(1024),
    };
    kv.put("bench", &value).await?;
    Response::builder().ok(value)
}

/// Makes 10 parallel sub-requests to `/hello` through the service binding to this worker.
async fn handle_fanout(env: &Env) -> Result<Response> {
    let service = env.service();
    let tasks = (0..10).map(|_| async {
        let res = service.fetch_with_str("http://bench/hello").await?;
        res.text().await
    });
    let bodies = futures_util::future::try_join_all(tasks).await?;
    let total: usize = bodies.iter().map(String::len).sum();
    Response::builder().ok(total.to_string())
}

/// Streams 1MB of data in 8KB chunks.
fn handle_stream() -> Result<Response> {
    let chunk = vec![b'x'; 8192];
    let chunks = (1024 * 1024) / chunk.len();
    let data_stream =
        futures_util::stream::iter((0..chunks).map(move |_| Ok::<Vec<u8>, Error>(chunk.clone())));
    Response::builder().from_stream(data_stream)
}
