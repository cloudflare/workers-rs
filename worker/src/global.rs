use edgeworker_sys::{Response as EdgeResponse, WorkerGlobalScope};
use crate::{Request as WorkerRequest, Response as WorkerResponse, Result};

use wasm_bindgen::{JsCast};
use wasm_bindgen_futures::JsFuture;

pub async fn fetch_with_str(url: &str) -> Result<WorkerResponse> {
    let worker: WorkerGlobalScope = js_sys::global().unchecked_into();
    let promise = worker.fetch_with_str(url);
    let resp = JsFuture::from(promise).await?;
    let resp: EdgeResponse = resp.dyn_into()?;
    Ok(resp.into())
}

pub async fn fetch_with_request(request: &WorkerRequest) -> Result<WorkerResponse> {
    let worker: WorkerGlobalScope = js_sys::global().unchecked_into();
    let req = request.inner();
    let promise = worker.fetch_with_request(req);
    let resp = JsFuture::from(promise).await?;
    let edge_response: EdgeResponse = resp.dyn_into()?;
    Ok(edge_response.into())
}