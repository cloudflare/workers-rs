use std::ops::Deref;

use crate::send::SendFuture;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use crate::{request::Request, response::Response, AbortSignal, Result};

/// Construct a Fetch call from a URL string or a Request object. Call its `send` method to execute
/// the request.
#[derive(Debug)]
pub enum Fetch {
    Url(url::Url),
    Request(Request),
}

impl Fetch {
    /// Execute a Fetch call and receive a Response.
    pub async fn send(&self) -> Result<Response> {
        match self {
            Fetch::Url(url) => fetch_with_str(url.as_ref(), None).await,
            Fetch::Request(req) => fetch_with_request(req, None).await,
        }
    }

    /// Execute a Fetch call and receive a Response.
    pub async fn send_with_signal(&self, signal: &AbortSignal) -> Result<Response> {
        match self {
            Fetch::Url(url) => fetch_with_str(url.as_ref(), Some(signal)).await,
            Fetch::Request(req) => fetch_with_request(req, Some(signal)).await,
        }
    }
}

async fn fetch_with_str(url: &str, signal: Option<&AbortSignal>) -> Result<Response> {
    let init = web_sys::RequestInit::new();
    init.set_signal(signal.map(|x| x.deref()));

    let worker: web_sys::WorkerGlobalScope = js_sys::global().unchecked_into();
    let promise = worker.fetch_with_str_and_init(url, &init);
    let resp = SendFuture::new(JsFuture::from(promise)).await?;
    let resp: web_sys::Response = resp.dyn_into()?;
    Ok(resp.into())
}

async fn fetch_with_request(request: &Request, signal: Option<&AbortSignal>) -> Result<Response> {
    let init = web_sys::RequestInit::new();
    init.set_signal(signal.map(|x| x.deref()));

    let worker: web_sys::WorkerGlobalScope = js_sys::global().unchecked_into();
    let req = request.inner();
    let promise = worker.fetch_with_request_and_init(req, &init);
    let resp = SendFuture::new(JsFuture::from(promise)).await?;
    let edge_response: web_sys::Response = resp.dyn_into()?;
    Ok(edge_response.into())
}
