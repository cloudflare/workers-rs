use wasm_bindgen::prelude::*;

use crate::types::{R2HttpMetadata, R2Range};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2Object;

    #[wasm_bindgen(method, catch, getter)]
    pub fn key(this: &R2Object) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn version(this: &R2Object) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn size(this: &R2Object) -> Result<u32, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn etag(this: &R2Object) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=httpEtag)]
    pub fn http_etag(this: &R2Object) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn uploaded(this: &R2Object) -> Result<js_sys::Date, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=httpMetadata)]
    pub fn http_metadata(this: &R2Object) -> Result<R2HttpMetadata, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn checksums(this: &R2Object) -> Result<js_sys::Object, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=customMetadata)]
    pub fn custom_metadata(this: &R2Object) -> Result<js_sys::Object, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn range(this: &R2Object) -> Result<R2Range, JsValue>;

    #[wasm_bindgen(method, catch, js_name=writeHttpMetadata)]
    pub fn write_http_metadata(
        this: &R2Object,
        headers: web_sys::Headers,
    ) -> Result<js_sys::Object, JsValue>;
}
