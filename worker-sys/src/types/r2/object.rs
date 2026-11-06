use wasm_bindgen::prelude::*;

use crate::types::R2HttpMetadata;

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
    pub fn size(this: &R2Object) -> Result<f64, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn etag(this: &R2Object) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=httpEtag)]
    pub fn http_etag(this: &R2Object) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn uploaded(this: &R2Object) -> Result<js_sys::Date, JsValue>;

    /// Absent (`undefined`) on objects returned by `list()` unless `httpMetadata` was requested
    /// through `include` (`r2_list_honor_include` compatibility flag, 2022-08-04).
    #[wasm_bindgen(method, catch, getter, js_name=httpMetadata)]
    pub fn http_metadata(this: &R2Object) -> Result<Option<R2HttpMetadata>, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn checksums(this: &R2Object) -> Result<js_sys::Object, JsValue>;

    /// Absent (`undefined`) on objects returned by `list()` unless `customMetadata` was requested
    /// through `include` (`r2_list_honor_include` compatibility flag, 2022-08-04).
    #[wasm_bindgen(method, catch, getter, js_name=customMetadata)]
    pub fn custom_metadata(this: &R2Object) -> Result<Option<js_sys::Object>, JsValue>;

    /// The range the object was read with, echoed by the runtime as a plain
    /// `{ offset, length }` object (`undefined` when the read had no range). It is not an
    /// `R2Range` instance, so it is read as a `JsValue` and decoded by the caller.
    #[wasm_bindgen(method, catch, getter)]
    pub fn range(this: &R2Object) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch, js_name=writeHttpMetadata)]
    pub fn write_http_metadata(
        this: &R2Object,
        headers: web_sys::Headers,
    ) -> Result<js_sys::Object, JsValue>;
}
