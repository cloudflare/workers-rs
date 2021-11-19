use serde::Serialize;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use worker_sys::cache::Cache as EdgeCache;
use worker_sys::Response as EdgeResponse;
use worker_sys::WorkerGlobalScope;

use crate::request::Request;
use crate::response::Response;
use crate::Result;

pub struct Cache {
    inner: EdgeCache,
}

impl Default for Cache {
    fn default() -> Self {
        let global: WorkerGlobalScope = js_sys::global().unchecked_into();

        Self {
            inner: global.caches().get_default_cache(),
        }
    }
}

impl Cache {
    pub async fn open(name: String) -> Self {
        let global: WorkerGlobalScope = js_sys::global().unchecked_into();
        let cache = global.caches().get_cache_from_name(name);

        // unwrap is safe because this promise never rejects
        // https://developer.mozilla.org/en-US/docs/Web/API/CacheStorage/open
        let inner = JsFuture::from(cache).await.unwrap().into();

        Self { inner }
    }

    pub async fn put(&self, request: &Request, response: Response) -> Result<()> {
        // TODO take the request by value and get the inner via From. for some reason that isn't working so...here we are
        // i don't think it's actually too expensive? like it's just cloning a pointer i think.
        let ffi_request = request.inner().clone()?;
        let ffi_response = response.into();
        let result = self.inner.put(ffi_request, ffi_response);
        let _ = JsFuture::from(result).await?;
        Ok(())
    }

    pub async fn r#match(&self, request: &Request, ignore_method: bool) -> Result<Response> {
        let options = JsValue::from_serde(&MatchOptions { ignore_method })?;
        let ffi_request = request.inner().clone()?;
        let result = self.inner.r#match(ffi_request, options);
        let edge_response: EdgeResponse = JsFuture::from(result).await?.into();
        let response = Response::from(edge_response);

        Ok(response)
    }

    pub async fn delete(&self, request: &Request, ignore_method: bool) -> Result<bool> {
        let options = JsValue::from_serde(&MatchOptions { ignore_method })?;
        let ffi_request = request.inner().clone()?;
        let result = JsFuture::from(self.inner.delete(ffi_request, options)).await?;
        // unwrap is safe because we know this is a boolean
        // https://developers.cloudflare.com/workers/runtime-apis/cache#delete
        Ok(result.as_bool().unwrap())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MatchOptions {
    ignore_method: bool,
}
