use crate::env::EnvBinding;
use crate::{Request, Response, Result};

use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use worker_sys::{RemoteService as EdgeRemoteService, Response as EdgeResponse};

/// Service bindings are an API that facilitate Worker-to-Worker
/// communication via explicit bindings defined in your configuration.
pub struct RemoteService {
    inner: EdgeRemoteService,
}

impl RemoteService {
    /// Send an internal Request to the Remote Service.
    pub async fn fetch_with_request(&self, req: Request) -> Result<Response> {
        let promise = self.inner.fetch_with_request_internal(req.inner());
        let response = JsFuture::from(promise).await?;
        Ok(response.dyn_into::<EdgeResponse>()?.into())
    }

    /// Construct a Request from a URL to the Remote Service.
    pub async fn fetch_with_str(&self, url: &str) -> Result<Response> {
        let promise = self.inner.fetch_with_str_internal(url);
        let response = JsFuture::from(promise).await?;
        Ok(response.dyn_into::<EdgeResponse>()?.into())
    }
}

impl EnvBinding for RemoteService {
    const TYPE_NAME: &'static str = "Fetcher";
}

impl JsCast for RemoteService {
    fn instanceof(val: &JsValue) -> bool {
        val.is_instance_of::<EdgeRemoteService>()
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        Self { inner: val.into() }
    }

    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}

impl From<RemoteService> for JsValue {
    fn from(ns: RemoteService) -> Self {
        JsValue::from(ns.inner)
    }
}

impl AsRef<JsValue> for RemoteService {
    fn as_ref(&self) -> &JsValue {
        &self.inner
    }
}
