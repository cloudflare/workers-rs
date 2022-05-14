use crate::Request as EdgeRequest;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen (extends = ::js_sys::Object, js_name = Fetcher)]
    pub type RemoteService;

    #[wasm_bindgen (method, js_class = "Fetcher", js_name = fetch)]
    pub fn fetch_with_request_internal(
        this: &RemoteService,
        req: &EdgeRequest,
    ) -> ::js_sys::Promise;

    #[wasm_bindgen (method, js_class = "Fetcher", js_name = fetch)]
    pub fn fetch_with_str_internal(this: &RemoteService, url: &str) -> ::js_sys::Promise;
}
