use crate::Request;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen (extends = ::js_sys::Object, js_name = Fetcher)]
    pub type Service;

    #[wasm_bindgen (method, js_class = "Fetcher", js_name = fetch)]
    pub fn fetch_with_request(this: &Service, req: &Request) -> ::js_sys::Promise;

    #[wasm_bindgen (method, js_class = "Fetcher", js_name = fetch)]
    pub fn fetch_with_url(this: &Service, url: &str) -> ::js_sys::Promise;
}
