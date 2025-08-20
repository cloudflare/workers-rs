use js_sys::Promise;
use wasm_bindgen::prelude::*;
use worker_kv::*;

async fn list() -> Result<JsValue, KvError> {
    let kv = KvStore::create("EXAMPLE")?;
    let list_response = kv.list().limit(100).execute().await?;

    // Returns a pretty printed version of the listed key value pairs.
    serde_json::to_string_pretty(&list_response)
        .map(Into::into)
        .map_err(Into::into)
}

#[wasm_bindgen]
pub fn start() -> Promise {
    wasm_bindgen_futures::future_to_promise(async { list().await.map_err(Into::into) })
}
