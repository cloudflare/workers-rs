use crate::{Request, Response};
use wasm_bindgen::prelude::*;

// An instance of cache
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = Cache)]
    #[derive(Debug, Clone, PartialEq)]
    pub type Cache;

    #[wasm_bindgen(structural, method, js_class=Cache, js_name = put)]
    pub fn put(this: &Cache, request: Request, response: Response) -> ::js_sys::Promise;

    #[wasm_bindgen(structural, method, js_class=Cache, js_name=match)]
    pub fn r#match(this: &Cache, request: Request, options: JsValue) -> ::js_sys::Promise;

    #[wasm_bindgen(structural, method, js_class=Cache, js_name=delete)]
    pub fn delete(this: &Cache, request: Request, options: JsValue) -> ::js_sys::Promise;
}

// `caches` global object
#[wasm_bindgen]
extern "C" {

    #[wasm_bindgen(js_name = Caches)]
    pub type Caches;

    #[wasm_bindgen(method, structural, getter, js_class = "Caches", js_name = default)]
    pub fn get_default_cache(this: &Caches) -> Cache;

    #[wasm_bindgen(method, structural, js_class = "Caches", js_name = open)]
    pub fn get_cache_from_name(this: &Caches, cache_name: String) -> ::js_sys::Promise;

}
