use crate::{request::Request as WorkerRequest, response::Response as WorkerResponse, Result};
use worker_sys::{Response as EdgeResponse, WorkerGlobalScope};

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

/// Construct a Fetch call from a URL string or a Request object. Call its `send` method to execute
/// the request.
pub enum Fetch {
    Url(url::Url),
    Request(WorkerRequest),
}

impl Fetch {
    /// Execute a Fetch call and receive a Response.
    pub async fn send(&self) -> Result<WorkerResponse> {
        match self {
            Fetch::Url(url) => fetch_with_str(url.as_ref()).await,
            Fetch::Request(req) => fetch_with_request(req).await,
        }
    }
}

async fn fetch_with_str(url: &str) -> Result<WorkerResponse> {
    let worker: WorkerGlobalScope = js_sys::global().unchecked_into();
    let promise = worker.fetch_with_str(url);
    let resp = JsFuture::from(promise).await?;
    let resp: EdgeResponse = resp.dyn_into()?;
    Ok(resp.into())
}

async fn fetch_with_request(request: &WorkerRequest) -> Result<WorkerResponse> {
    let worker: WorkerGlobalScope = js_sys::global().unchecked_into();
    let req = request.inner();
    let promise = worker.fetch_with_request(req);
    let resp = JsFuture::from(promise).await?;
    let edge_response: EdgeResponse = resp.dyn_into()?;
    Ok(edge_response.into())
}
