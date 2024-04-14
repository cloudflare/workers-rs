use crate::{env::EnvBinding, RequestInit, Result};
#[cfg(feature = "http")]
use std::convert::TryInto;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

#[cfg(feature = "http")]
use crate::{HttpRequest, HttpResponse};
use crate::{Request, Response};
/// A struct for invoking fetch events to other Workers.
#[derive(Clone)]
pub struct Fetcher(worker_sys::Fetcher);

#[cfg(not(feature = "http"))]
type FetchResponseType = Response;
#[cfg(feature = "http")]
type FetchResponseType = HttpResponse;

#[cfg(not(feature = "http"))]
type FetchRequestType = Request;
#[cfg(feature = "http")]
type FetchRequestType = HttpRequest;

impl Fetcher {
    /// Invoke a fetch event in a worker with a url and optionally a [RequestInit].
    ///
    /// Return type is [`Response`](crate::Response) unless `http` feature is enabled
    /// and then it is [`http::Response<worker::Body>`].
    pub async fn fetch(
        &self,
        url: impl Into<String>,
        init: Option<RequestInit>,
    ) -> Result<FetchResponseType> {
        let path = url.into();
        let promise = match init {
            Some(ref init) => self.0.fetch_with_str_and_init(&path, &init.into()),
            None => self.0.fetch_with_str(&path),
        }?;

        let resp_sys: web_sys::Response = JsFuture::from(promise).await?.dyn_into()?;
        #[cfg(not(feature = "http"))]
        let result = Ok(Response::from(resp_sys));
        #[cfg(feature = "http")]
        let result = crate::response_from_wasm(resp_sys);
        result
    }

    /// Invoke a fetch event with an existing [Request].
    ///
    /// Argument type is [`Request`](crate::Request) unless `http` feature is enabled
    /// and then it is [`http::Request<worker::Body>`].
    ///
    /// Return type is [`Response`](crate::Response) unless `http` feature is enabled
    /// and then it is [`http::Response<worker::Body>`].
    pub async fn fetch_request(&self, request: FetchRequestType) -> Result<FetchResponseType> {
        #[cfg(feature = "http")]
        let req = TryInto::<Request>::try_into(request)?;
        #[cfg(not(feature = "http"))]
        let req = request;
        let promise = self.0.fetch(req.inner())?;
        let resp_sys: web_sys::Response = JsFuture::from(promise).await?.dyn_into()?;
        let response = Response::from(resp_sys);
        #[cfg(feature = "http")]
        let result = response.try_into();
        #[cfg(not(feature = "http"))]
        let result = Ok(response);
        result
    }
}

impl EnvBinding for Fetcher {
    const TYPE_NAME: &'static str = "Fetcher";
}

impl JsCast for Fetcher {
    fn instanceof(val: &wasm_bindgen::JsValue) -> bool {
        val.is_instance_of::<Fetcher>()
    }

    fn unchecked_from_js(val: wasm_bindgen::JsValue) -> Self {
        Self(val.into())
    }

    fn unchecked_from_js_ref(val: &wasm_bindgen::JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}

impl From<Fetcher> for JsValue {
    fn from(service: Fetcher) -> Self {
        JsValue::from(service.0)
    }
}

impl AsRef<wasm_bindgen::JsValue> for Fetcher {
    fn as_ref(&self) -> &wasm_bindgen::JsValue {
        &self.0
    }
}

impl From<worker_sys::Fetcher> for Fetcher {
    fn from(inner: worker_sys::Fetcher) -> Self {
        Self(inner)
    }
}
