use wasm_bindgen::prelude::*;

use crate::types::R2Object;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=R2Object, js_name=R2ObjectBody)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2ObjectBody;

    #[wasm_bindgen(structural, method, getter, js_class=R2ObjectBody, js_name=body)]
    pub fn body(this: &R2ObjectBody) -> web_sys::ReadableStream;

    #[wasm_bindgen(structural, method, getter, js_class=R2ObjectBody, js_name=bodyUsed)]
    pub fn body_used(this: &R2ObjectBody) -> bool;

    #[wasm_bindgen(structural, method, js_class=R2ObjectBody, js_name=arrayBuffer)]
    pub fn array_buffer(this: &R2ObjectBody) -> js_sys::Promise;
}
