use std::ops::Deref;

use crate::SendJsFuture;
use wasm_bindgen::JsCast;

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
    let fut = {
        let mut init = web_sys::RequestInit::new();
        init.signal(signal.map(|x| x.deref()));
        let worker: web_sys::WorkerGlobalScope = js_sys::global().unchecked_into();
        let promise = worker.fetch_with_str_and_init(url, &init);
        SendJsFuture::from(promise)
    };
    let resp = fut.await?;
    let resp: web_sys::Response = resp.dyn_into()?;
    Ok(resp.into())
}

async fn fetch_with_request(
    request: &WorkerRequest,
    signal: Option<&AbortSignal>,
) -> Result<WorkerResponse> {
    let req = request.inner();

    let fut = {
        let mut init = web_sys::RequestInit::new();
        init.signal(signal.map(|x| x.deref()));
        let worker: web_sys::WorkerGlobalScope = js_sys::global().unchecked_into();
        let promise = worker.fetch_with_request_and_init(req, &init);
        SendJsFuture::from(promise)
    };
    let resp = fut.await?;
    let edge_response: web_sys::Response = resp.dyn_into()?;
    Ok(edge_response.into())
}
