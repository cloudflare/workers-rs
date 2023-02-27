use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

use crate::{
    body::Body,
    env::EnvBinding,
    http::{request, response},
    Result,
};

/// A struct for invoking fetch events to other Workers.
pub struct Fetcher(worker_sys::Fetcher);

impl Fetcher {
    /// Invoke a fetch event in a worker with a url and optionally a [RequestInit].
    pub async fn fetch(&self, req: http::Request<Body>) -> Result<http::Response<Body>> {
        let req = request::into_wasm(req);
        let promise = self.0.fetch(&req);

        let res = JsFuture::from(promise).await?.dyn_into()?;
        Ok(response::from_wasm(res))
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
