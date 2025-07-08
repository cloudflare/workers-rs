use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    LazyLock, Mutex,
};
#[cfg(feature = "http")]
use tower_service::Service;
#[cfg(feature = "http")]
use worker::HttpRequest;
use worker::{console_log, event, js_sys, wasm_bindgen, Env, Result};
#[cfg(not(feature = "http"))]
use worker::{Request, Response};

mod alarm;
mod analytics_engine;
mod assets;
mod auto_response;
mod cache;
mod counter;
mod d1;
mod durable;
mod fetch;
mod form;
mod js_snippets;
mod kv;
mod put_raw;
mod queue;
mod r2;
mod request;
mod router;
mod service;
mod socket;
mod sql_counter;
mod sql_iterator;
mod user;
mod utils;
mod ws;

#[derive(Deserialize, Serialize)]
struct MyData {
    message: String,
    #[serde(default)]
    is: bool,
    #[serde(default)]
    data: Vec<u8>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiData {
    user_id: i32,
    title: String,
    completed: bool,
}

#[derive(Clone)]
pub struct SomeSharedData {
    regex: &'static Regex,
}

static GLOBAL_STATE: AtomicBool = AtomicBool::new(false);

static GLOBAL_QUEUE_STATE: Mutex<Vec<queue::QueueBody>> = Mutex::new(Vec::new());

static DATA_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap());

// We're able to specify a start event that is called when the WASM is initialized before any
// requests. This is useful if you have some global state or setup code, like a logger. This is
// only called once for the entire lifetime of the worker.
#[event(start)]
pub fn start() {
    utils::set_panic_hook();

    // Change some global state so we know that we ran our setup function.
    GLOBAL_STATE.store(true, Ordering::SeqCst);
}

#[cfg(feature = "http")]
type HandlerRequest = HttpRequest;
#[cfg(not(feature = "http"))]
type HandlerRequest = Request;
#[cfg(feature = "http")]
type HandlerResponse = http::Response<axum::body::Body>;
#[cfg(not(feature = "http"))]
type HandlerResponse = Response;

/// Entrypoint to the worker for handling fetch requests.
///
/// # Errors
///
/// Returns the same error as the underlying router implementation.
#[event(fetch, respond_with_errors)]
pub async fn main(
    request: HandlerRequest,
    env: Env,
    _ctx: worker::Context,
) -> Result<HandlerResponse> {
    let data = SomeSharedData { regex: &DATA_REGEX };

    #[cfg(feature = "http")]
    let res = {
        let mut router = router::make_router(data, env);
        Ok(Service::call(&mut router, request).await?)
    };

    #[cfg(not(feature = "http"))]
    let res = {
        let router = router::make_router(data);
        router.run(request, env).await
    };

    res
}
