use js_sys::Promise;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

use crate::{env::EnvBinding, Request, Response, Result};

pub struct Service {
    service: worker_sys::Service,
}

async fn as_response(promise: Promise) -> Result<Response> {
    let response = JsFuture::from(promise).await?;
    Ok(response.dyn_into::<worker_sys::Response>()?.into())
}

impl Service {
    pub async fn fetch_with_request(&self, req: Request) -> Result<Response> {
        let promise = self.service.fetch_with_request(req.inner());
        as_response(promise).await
    }

    pub async fn fetch_with_url(&self, url: &str) -> Result<Response> {
        let promise = self.service.fetch_with_url(url);
        as_response(promise).await
    }
}

impl JsCast for Service {
    fn instanceof(val: &wasm_bindgen::JsValue) -> bool {
        val.is_instance_of::<Service>()
    }

    fn unchecked_from_js(val: wasm_bindgen::JsValue) -> Self {
        Self {
            service: val.into(),
        }
    }

    fn unchecked_from_js_ref(val: &wasm_bindgen::JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}

impl From<Service> for JsValue {
    fn from(service: Service) -> Self {
        JsValue::from(service.service)
    }
}

impl AsRef<wasm_bindgen::JsValue> for Service {
    fn as_ref(&self) -> &wasm_bindgen::JsValue {
        &self.service
    }
}

impl EnvBinding for Service {
    const TYPE_NAME: &'static str = "Fetcher";
}
