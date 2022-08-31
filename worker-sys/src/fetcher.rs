use wasm_bindgen::prelude::*;

use crate::{Request, RequestInit};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = ::js_sys::Object, js_name = Fetcher)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Fetcher;

    #[wasm_bindgen(structural, method, js_class=Fetcher, js_name = fetch)]
    pub fn fetch(this: &Fetcher, input: &Request) -> ::js_sys::Promise;
    #[wasm_bindgen(structural, method, js_class=Fetcher, js_name = fetch)]
    pub fn fetch_with_str(this: &Fetcher, input: &str) -> ::js_sys::Promise;
    #[wasm_bindgen(structural, method, js_class=Fetcher, js_name = fetch)]
    pub fn fetch_with_init(
        this: &Fetcher,
        input: &Request,
        init: &RequestInit,
    ) -> ::js_sys::Promise;
    #[wasm_bindgen(structural, method, js_class=Fetcher, js_name = fetch)]
    pub fn fetch_with_str_and_init(
        this: &Fetcher,
        input: &str,
        init: &RequestInit,
    ) -> ::js_sys::Promise;
}
