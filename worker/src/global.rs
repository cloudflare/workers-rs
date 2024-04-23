use std::ops::Deref;

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use crate::{AbortSignal, Result};

#[cfg(feature = "http")]
use crate::{HttpRequest, HttpResponse};
use crate::{Request, Response};

#[cfg(not(feature = "http"))]
type FetchResponseType = Response;
#[cfg(feature = "http")]
type FetchResponseType = HttpResponse;

#[cfg(not(feature = "http"))]
type FetchRequestType = Request;
#[cfg(feature = "http")]
type FetchRequestType = HttpRequest;

/// Construct a Fetch call from a URL string or a Request object. Call its `send` method to execute
/// the request.
pub enum Fetch {
    Url(url::Url),
    Request(FetchRequestType),
}

impl Fetch {
    /// Execute a Fetch call and receive a Response.
    pub async fn send(&self) -> Result<FetchResponseType> {
        match self {
            Fetch::Url(url) => fetch_with_str(url.as_ref(), None).await,
            Fetch::Request(req) => fetch_with_request(req, None).await,
        }
    }

    /// Execute a Fetch call and receive a Response.
    pub async fn send_with_signal(&self, signal: &AbortSignal) -> Result<FetchResponseType> {
        match self {
            Fetch::Url(url) => fetch_with_str(url.as_ref(), Some(signal)).await,
            Fetch::Request(req) => fetch_with_request(req, Some(signal)).await,
        }
    }
}

async fn fetch_with_str(url: &str, signal: Option<&AbortSignal>) -> Result<FetchResponseType> {
    let mut init = web_sys::RequestInit::new();
    init.signal(signal.map(|x| x.deref()));

    let worker: web_sys::WorkerGlobalScope = js_sys::global().unchecked_into();
    let promise = worker.fetch_with_str_and_init(url, &init);
    let resp: web_sys::Response = JsFuture::from(promise).await?.dyn_into()?;
    #[cfg(not(feature = "http"))]
    let result = Ok(Response::from(resp));
    #[cfg(feature = "http")]
    let result = crate::response_from_wasm(resp_sys);
    result
}

async fn fetch_with_request(
    request: &FetchRequestType,
    signal: Option<&AbortSignal>,
) -> Result<FetchResponseType> {
    let mut init = web_sys::RequestInit::new();
    init.signal(signal.map(|x| x.deref()));
    #[cfg(feature = "http")]
    let req = TryInto::<Request>::try_into(request)?;
    #[cfg(not(feature = "http"))]
    let req = request;

    let worker: web_sys::WorkerGlobalScope = js_sys::global().unchecked_into();
    let req = req.inner();
    let promise = worker.fetch_with_request_and_init(req, &init);
    let resp_sys: web_sys::Response = JsFuture::from(promise).await?.dyn_into()?;
    let response = Response::from(resp_sys);

    #[cfg(feature = "http")]
    let result = response.try_into();
    #[cfg(not(feature = "http"))]
    let result = Ok(response);

    result
}
