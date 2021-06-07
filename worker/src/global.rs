use edgeworker_sys::{Response as EdgeResponse, WorkerGlobalScope};

use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

pub async fn fetch_with_str(url: &str) -> Result<EdgeResponse, JsValue> {
    let worker: WorkerGlobalScope = js_sys::global().unchecked_into();
    let promise = worker.fetch_with_str(url);
    let resp = JsFuture::from(promise).await?;
    let resp: EdgeResponse = resp.dyn_into().unwrap();
    Ok(resp)
}
