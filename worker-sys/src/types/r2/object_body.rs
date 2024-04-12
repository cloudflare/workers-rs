use wasm_bindgen::prelude::*;

use crate::types::R2Object;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=R2Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2ObjectBody;

    #[wasm_bindgen(method, catch, getter)]
    pub fn body(this: &R2ObjectBody) -> Result<web_sys::ReadableStream, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=bodyUsed)]
    pub fn body_used(this: &R2ObjectBody) -> Result<bool, JsValue>;

    #[wasm_bindgen(method, catch, js_name=arrayBuffer)]
    pub fn array_buffer(this: &R2ObjectBody) -> Result<js_sys::Promise, JsValue>;
}
