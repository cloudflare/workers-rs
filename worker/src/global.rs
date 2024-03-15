use std::ops::Deref;

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use crate::{AbortSignal, Result};

#[cfg(feature = "http")]
type WorkerRequest = http::Request<crate::Body>;
#[cfg(not(feature = "http"))]
type WorkerRequest = crate::request::Request;

#[cfg(feature = "http")]
type WorkerResponse = http::Response<crate::Body>;
#[cfg(not(feature = "http"))]
type WorkerResponse = crate::response::Response;

/// Construct a Fetch call from a URL string or a Request object. Call its `send` method to execute
/// the request.
pub enum Fetch {
    Url(url::Url),
    Request(WorkerRequest),
}

#[cfg(not(feature = "http"))]
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

#[cfg(feature = "http")]
impl Fetch {
    /// Execute a Fetch call and receive a Response.
    pub async fn send(self) -> Result<WorkerResponse> {
        match self {
            Fetch::Url(url) => fetch_with_str(url.as_ref(), None).await,
            Fetch::Request(req) => fetch_with_request(req, None).await,
        }
    }

    /// Execute a Fetch call and receive a Response.
    pub async fn send_with_signal(self, signal: &AbortSignal) -> Result<WorkerResponse> {
        match self {
            Fetch::Url(url) => fetch_with_str(url.as_ref(), Some(signal)).await,
            Fetch::Request(req) => fetch_with_request(req, Some(signal)).await,
        }
    }
}

async fn fetch_with_str(url: &str, signal: Option<&AbortSignal>) -> Result<WorkerResponse> {
    let mut init = web_sys::RequestInit::new();
    init.signal(signal.map(|x| x.deref()));

    let worker: web_sys::WorkerGlobalScope = js_sys::global().unchecked_into();
    let promise = worker.fetch_with_str_and_init(url, &init);
    let resp = JsFuture::from(promise).await?;
    let resp: web_sys::Response = resp.dyn_into()?;
    #[cfg(feature = "http")]
    let result = crate::response_from_wasm(resp);
    #[cfg(not(feature = "http"))]
    let result = Ok(resp.into());
    result
}

async fn fetch_with_request(
    #[cfg(feature = "http")] request: WorkerRequest,
    #[cfg(not(feature = "http"))] request: &WorkerRequest,
    signal: Option<&AbortSignal>,
) -> Result<WorkerResponse> {
    let mut init = web_sys::RequestInit::new();
    init.signal(signal.map(|x| x.deref()));

    let worker: web_sys::WorkerGlobalScope = js_sys::global().unchecked_into();

    #[cfg(feature = "http")]
    let req = &crate::request_to_wasm(request)?;
    #[cfg(not(feature = "http"))]
    let req = request.inner();

    let promise = worker.fetch_with_request_and_init(req, &init);
    let js_resp = JsFuture::from(promise).await?;
    let resp: web_sys::Response = js_resp.dyn_into()?;
    #[cfg(feature = "http")]
    let result = crate::response_from_wasm(resp);
    #[cfg(not(feature = "http"))]
    let result = Ok(resp.into());
    result
}
