use std::ops::Deref;
use std::time::Duration;

use reqwest::{Body, Client, Error, Response};
use tokio::time;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use crate::{
     AbortSignal, Result,
};

/// Construct a Fetch call from a URL string or a Request object. Call its `send` method to execute
/// the request.
pub enum Fetch {
    Url(url::Url),
    Request(Body),
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

//#[cfg(feature = "http")]
async fn fetch_with_str(url: &str, signal: Option<&AbortSignal>) -> Result<Response> {
    let client = Client::new();
    let request_future = client.get(url).send();
    let timeout_future = time::sleep(Duration::from_secs(10)); // TODO: Remove 10 seconds magic value.
    tokio::select! {
        result = request_future => {
            // The request completed successfully
            result.map_err(|err| err.into())
        },
        _ = timeout_future => {
            // The timeout occurred before the request completed
            Err(reqwest::Error::new(reqwest::StatusCode::REQUEST_TIMEOUT, "Request timed out"))
        }
    }
}

//#[cfg(feature = "http")]
async fn fetch_with_request(
    request: &Response,
    signal: Option<&AbortSignal>,
) -> Result<Response> {
    let mut init = web_sys::RequestInit::new();
    init.signal(signal.map(|x| x.deref()));

    let worker: web_sys::WorkerGlobalScope = js_sys::global().unchecked_into();
    let req = request.inner();
    let promise = worker.fetch_with_request_and_init(req, &init);
    let resp = JsFuture::from(promise).await?;
    let edge_response: web_sys::Response = resp.dyn_into()?;
    Ok(edge_response.into())
}

/*
#[cfg(not(feature = "http"))]
async fn fetch_with_str(url: &str, signal: Option<&AbortSignal>) -> Result<WorkerResponse> {
    let mut init = web_sys::RequestInit::new();
    init.signal(signal.map(|x| x.deref()));

    let worker: web_sys::WorkerGlobalScope = js_sys::global().unchecked_into();
    let promise = worker.fetch_with_str_and_init(url, &init);
    let resp = JsFuture::from(promise).await?;
    let resp: web_sys::Response = resp.dyn_into()?;
    Ok(resp.into())
}

#[cfg(not(feature = "http"))]
async fn fetch_with_request(
    request: &WorkerRequest,
    signal: Option<&AbortSignal>,
) -> Result<WorkerResponse> {
    let mut init = web_sys::RequestInit::new();
    init.signal(signal.map(|x| x.deref()));

    let worker: web_sys::WorkerGlobalScope = js_sys::global().unchecked_into();
    let req = request.inner();
    let promise = worker.fetch_with_request_and_init(req, &init);
    let resp = JsFuture::from(promise).await?;
    let edge_response: web_sys::Response = resp.dyn_into()?;
    Ok(edge_response.into())
}*/
