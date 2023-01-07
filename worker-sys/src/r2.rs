use js_sys::{JsString, Object};
use wasm_bindgen::prelude::*;
use web_sys::ReadableStream;

use crate::Headers;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=R2Bucket)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2Bucket;

    #[wasm_bindgen(structural, method, js_class=R2Bucket, js_name = head)]
    pub fn head(this: &R2Bucket, key: String) -> ::js_sys::Promise;
    #[wasm_bindgen(structural, method, js_class=R2Bucket, js_name = get)]
    pub fn get(this: &R2Bucket, key: String, options: JsValue) -> ::js_sys::Promise;
    #[wasm_bindgen(structural, method, js_class=R2Bucket, js_name = put)]
    pub fn put(this: &R2Bucket, key: String, value: JsValue, options: JsValue)
        -> ::js_sys::Promise;
    #[wasm_bindgen(structural, method, js_class=R2Bucket, js_name = delete)]
    pub fn delete(this: &R2Bucket, key: String) -> ::js_sys::Promise;
    #[wasm_bindgen(structural, method, js_class=R2Bucket, js_name = list)]
    pub fn list(this: &R2Bucket, options: JsValue) -> ::js_sys::Promise;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=R2Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2Object;

    #[wasm_bindgen(structural, method, getter, js_class=R2Object, js_name = key)]
    pub fn key(this: &R2Object) -> String;
    #[wasm_bindgen(structural, method, getter, js_class=R2Object, js_name = version)]
    pub fn version(this: &R2Object) -> String;
    #[wasm_bindgen(structural, method, getter, js_class=R2Object, js_name = size)]
    pub fn size(this: &R2Object) -> u32;
    #[wasm_bindgen(structural, method, getter, js_class=R2Object, js_name = etag)]
    pub fn etag(this: &R2Object) -> String;
    #[wasm_bindgen(structural, method, getter, js_class=R2Object, js_name = httpEtag)]
    pub fn http_etag(this: &R2Object) -> String;
    #[wasm_bindgen(structural, method, getter, js_class=R2Object, js_name = uploaded)]
    pub fn uploaded(this: &R2Object) -> ::js_sys::Date;
    #[wasm_bindgen(structural, method, getter, js_class=R2Object, js_name = httpMetadata)]
    pub fn http_metadata(this: &R2Object) -> R2HttpMetadata;
    #[wasm_bindgen(structural, method, getter, js_class=R2Object, js_name = customMetadata)]
    pub fn custom_metadata(this: &R2Object) -> Object;
    #[wasm_bindgen(structural, method, getter, js_class=R2Object, js_name = range)]
    pub fn range(this: &R2Object) -> R2Range;
    #[wasm_bindgen(structural, method, js_class=R2Object, js_name = writeHttpMetadata, catch)]
    pub fn write_http_metadata(this: &R2Object, headers: Headers) -> Result<Object, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=R2Object, js_name=R2ObjectBody)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2ObjectBody;

    #[wasm_bindgen(structural, method, getter, js_class=R2ObjectBody, js_name = body)]
    pub fn body(this: &R2ObjectBody) -> ReadableStream;
    #[wasm_bindgen(structural, method, getter, js_class=R2ObjectBody, js_name = bodyUsed)]
    pub fn body_used(this: &R2ObjectBody) -> bool;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=R2Objects)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2Objects;

    #[wasm_bindgen(structural, method, getter, js_class=R2Objects, js_name = objects)]
    pub fn objects(this: &R2Objects) -> Vec<R2Object>;
    #[wasm_bindgen(structural, method, getter, js_class=R2Objects, js_name = truncated)]
    pub fn truncated(this: &R2Objects) -> bool;
    #[wasm_bindgen(structural, method, getter, js_class=R2Objects, js_name = cursor)]
    pub fn cursor(this: &R2Objects) -> Option<String>;
    #[wasm_bindgen(structural, method, getter, js_class=R2Objects, js_name = delimitedPrefixes)]
    pub fn delimited_prefixes(this: &R2Objects) -> Vec<JsString>;
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct R2Range {
    pub offset: Option<u32>,
    pub length: Option<u32>,
    pub suffix: Option<u32>,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=R2HttpMetadata)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2HttpMetadata;

    #[wasm_bindgen(structural, method, getter, js_class=R2HttpMetadata, js_name = contentType)]
    pub fn content_type(this: &R2HttpMetadata) -> Option<String>;
    #[wasm_bindgen(structural, method, getter, js_class=R2HttpMetadata, js_name = contentLanguage)]
    pub fn content_language(this: &R2HttpMetadata) -> Option<String>;
    #[wasm_bindgen(structural, method, getter, js_class=R2HttpMetadata, js_name = contentDisposition)]
    pub fn content_disposition(this: &R2HttpMetadata) -> Option<String>;
    #[wasm_bindgen(structural, method, getter, js_class=R2HttpMetadata, js_name = contentEncoding)]
    pub fn content_encoding(this: &R2HttpMetadata) -> Option<String>;
    #[wasm_bindgen(structural, method, getter, js_class=R2HttpMetadata, js_name = cacheControl)]
    pub fn cache_control(this: &R2HttpMetadata) -> Option<String>;
    #[wasm_bindgen(structural, method, getter, js_class=R2HttpMetadata, js_name = cacheExpiry)]
    pub fn cache_expiry(this: &R2HttpMetadata) -> Option<::js_sys::Date>;
}
