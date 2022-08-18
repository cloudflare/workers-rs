use wasm_bindgen::prelude::*;

use crate::{Request, Response};

// An instance of cache
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = ::js_sys::Object, js_name = Cache)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Cache;

    #[wasm_bindgen(structural, method, js_class=Cache, js_name = put)]
    pub fn put_request(this: &Cache, request: &Request, response: &Response) -> ::js_sys::Promise;

    #[wasm_bindgen(structural, method, js_class=Cache, js_name = put)]
    pub fn put_url(this: &Cache, url: &str, response: &Response) -> ::js_sys::Promise;

    #[wasm_bindgen(structural, method, js_class=Cache, js_name=match)]
    pub fn match_request(this: &Cache, request: &Request, options: JsValue) -> ::js_sys::Promise;

    #[wasm_bindgen(structural, method, js_class=Cache, js_name=match)]
    pub fn match_url(this: &Cache, url: &str, options: JsValue) -> ::js_sys::Promise;

    #[wasm_bindgen(structural, method, js_class=Cache, js_name=delete)]
    pub fn delete_request(this: &Cache, request: &Request, options: JsValue) -> ::js_sys::Promise;

    #[wasm_bindgen(structural, method, js_class=Cache, js_name=delete)]
    pub fn delete_url(this: &Cache, request: &str, options: JsValue) -> ::js_sys::Promise;
}

// `caches` global object
#[wasm_bindgen]
extern "C" {

    #[wasm_bindgen(js_name = Caches)]
    pub type Caches;

    #[wasm_bindgen(method, structural, getter, js_class = "Caches")]
    pub fn default(this: &Caches) -> Cache;

    #[wasm_bindgen(method, structural, js_class = "Caches")]
    pub fn open(this: &Caches, cache_name: String) -> ::js_sys::Promise;

}
