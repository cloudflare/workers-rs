use wasm_bindgen::prelude::*;

use crate::types::R2Object;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=R2Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2ObjectBody;

    #[wasm_bindgen(method, getter)]
    pub fn body(this: &R2ObjectBody) -> web_sys::ReadableStream;

    #[wasm_bindgen(method, getter, js_name=bodyUsed)]
    pub fn body_used(this: &R2ObjectBody) -> bool;

    #[wasm_bindgen(method, js_name=arrayBuffer)]
    pub fn array_buffer(this: &R2ObjectBody) -> js_sys::Promise;
}
