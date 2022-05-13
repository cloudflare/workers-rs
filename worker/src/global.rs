use std::ops::Deref;

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use worker_sys::{RequestInit as EdgeRequestInit, Response as EdgeResponse, WorkerGlobalScope};

use crate::{
    request::Request as WorkerRequest, response::Response as WorkerResponse, AbortSignal, Result,
};

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
            Fetch::Url(url) => fetch_with_str(url.as_ref(), None).await,
            Fetch::Request(req) => fetch_with_request(req, None).await,
        }
    }

    /// Execute a Fetch call and receive a Response.
    pub async fn send_with_signal(&self, signal: &AbortSignal) -> Result<WorkerResponse> {
        match self {
            Fetch::Url(url) => fetch_with_str(url.as_ref(), Some(signal)).await,
            Fetch::Request(req) => fetch_with_request(req, Some(signal)).await,
        }
    }
}

async fn fetch_with_str(url: &str, signal: Option<&AbortSignal>) -> Result<WorkerResponse> {
    let mut init = EdgeRequestInit::new();
    init.signal(signal.map(|x| x.deref()));

    let worker: WorkerGlobalScope = js_sys::global().unchecked_into();
    let promise = worker.fetch_with_str_and_init(url, &init);
    let resp = JsFuture::from(promise).await?;
    let resp: EdgeResponse = resp.dyn_into()?;
    Ok(resp.into())
}

async fn fetch_with_request(
    request: &WorkerRequest,
    signal: Option<&AbortSignal>,
) -> Result<WorkerResponse> {
    let mut init = EdgeRequestInit::new();
    init.signal(signal.map(|x| x.deref()));

    let worker: WorkerGlobalScope = js_sys::global().unchecked_into();
    let req = request.inner();
    let promise = worker.fetch_with_request_and_init(req, &init);
    let resp = JsFuture::from(promise).await?;
    let edge_response: EdgeResponse = resp.dyn_into()?;
    Ok(edge_response.into())
}
