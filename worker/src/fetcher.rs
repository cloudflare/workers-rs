use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use worker_sys::{Fetcher as FetcherSys, Response as ResponseSys};

use crate::{env::EnvBinding, Request, RequestInit, Response, Result};

/// A struct for invoking fetch events to other Workers.
pub struct Fetcher(FetcherSys);

impl Fetcher {
    /// Invoke a fetch event in a worker with a url and optionally a [RequestInit].
    pub async fn fetch(
        &self,
        url: impl Into<String>,
        init: Option<RequestInit>,
    ) -> Result<Response> {
        let path = url.into();
        let promise = match init {
            Some(ref init) => self.0.fetch_with_str_and_init(&path, &init.into()),
            None => self.0.fetch_with_str(&path),
        };

        let resp_sys: ResponseSys = JsFuture::from(promise).await?.dyn_into()?;
        Ok(Response::from(resp_sys))
    }

    /// Invoke a fetch event with an existing [Request].
    pub async fn fetch_request(&self, request: Request) -> Result<Response> {
        let promise = self.0.fetch(request.inner());
        let resp_sys: ResponseSys = JsFuture::from(promise).await?.dyn_into()?;
        Ok(Response::from(resp_sys))
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

impl From<FetcherSys> for Fetcher {
    fn from(inner: FetcherSys) -> Self {
        Self(inner)
    }
}
