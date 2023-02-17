use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object, js_name=R2HttpMetadata)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2HttpMetadata;

    #[wasm_bindgen(structural, method, getter, js_class=R2HttpMetadata, js_name=contentType)]
    pub fn content_type(this: &R2HttpMetadata) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=R2HttpMetadata, js_name=contentLanguage)]
    pub fn content_language(this: &R2HttpMetadata) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=R2HttpMetadata, js_name=contentDisposition)]
    pub fn content_disposition(this: &R2HttpMetadata) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=R2HttpMetadata, js_name=contentEncoding)]
    pub fn content_encoding(this: &R2HttpMetadata) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=R2HttpMetadata, js_name=cacheControl)]
    pub fn cache_control(this: &R2HttpMetadata) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=R2HttpMetadata, js_name=cacheExpiry)]
    pub fn cache_expiry(this: &R2HttpMetadata) -> Option<js_sys::Date>;
}
