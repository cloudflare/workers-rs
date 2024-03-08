use wasm_bindgen::{JsCast, JsValue};

use crate::{
    body::Body,
    env::EnvBinding,
    futures::SendJsFuture,
    http::{request, response},
    RequestInit,
    Result,
};

/// A struct for invoking fetch events to other Workers.
pub struct Fetcher(worker_sys::Fetcher);

unsafe impl Send for Fetcher {}
unsafe impl Sync for Fetcher {}

impl Fetcher {
    /// Invoke a fetch event in a worker with a url and optionally a [RequestInit].
    pub async fn fetch(
        &self,
        url: impl Into<String>,
        init: Option<RequestInit>,
    ) -> Result<http::Response<Body>> {
        let path = url.into();
        let fut = {
            let promise = match init {
                Some(ref init) => self.0.fetch_with_str_and_init(&path, &init.into()),
                None => self.0.fetch_with_str(&path),
            };

            SendJsFuture::from(promise)
        };

        let res = fut.await?.dyn_into()?;
        Ok(response::from_wasm(res))
    }


    /// Invoke a fetch event with an existing [Request].
    pub async fn fetch_request(&self, req: http::Request<Body>) -> Result<http::Response<Body>> {
        let fut = {
            let req = request::into_wasm(req);
            let promise = self.0.fetch(&req);

            SendJsFuture::from(promise)
        };

        let res = fut.await?.dyn_into()?;
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
