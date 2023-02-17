use wasm_bindgen::prelude::*;

use crate::types::{R2HttpMetadata, R2Range};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object, js_name=R2Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2Object;

    #[wasm_bindgen(structural, method, getter, js_class=R2Object, js_name=key)]
    pub fn key(this: &R2Object) -> String;

    #[wasm_bindgen(structural, method, getter, js_class=R2Object, js_name = version)]
    pub fn version(this: &R2Object) -> String;

    #[wasm_bindgen(structural, method, getter, js_class=R2Object, js_name=size)]
    pub fn size(this: &R2Object) -> u32;

    #[wasm_bindgen(structural, method, getter, js_class=R2Object, js_name=etag)]
    pub fn etag(this: &R2Object) -> String;

    #[wasm_bindgen(structural, method, getter, js_class=R2Object, js_name=httpEtag)]
    pub fn http_etag(this: &R2Object) -> String;

    #[wasm_bindgen(structural, method, getter, js_class=R2Object, js_name=uploaded)]
    pub fn uploaded(this: &R2Object) -> js_sys::Date;

    #[wasm_bindgen(structural, method, getter, js_class=R2Object, js_name=httpMetadata)]
    pub fn http_metadata(this: &R2Object) -> R2HttpMetadata;

    #[wasm_bindgen(structural, method, getter, js_class=R2Object, js_name=customMetadata)]
    pub fn custom_metadata(this: &R2Object) -> js_sys::Object;

    #[wasm_bindgen(structural, method, getter, js_class=R2Object, js_name=range)]
    pub fn range(this: &R2Object) -> R2Range;

    #[wasm_bindgen(structural, method, js_class=R2Object, js_name=writeHttpMetadata, catch)]
    pub fn write_http_metadata(
        this: &R2Object,
        headers: web_sys::Headers,
    ) -> Result<js_sys::Object, JsValue>;
}
