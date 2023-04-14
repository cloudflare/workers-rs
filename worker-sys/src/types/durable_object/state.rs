use wasm_bindgen::prelude::*;

use crate::types::{DurableObjectId, DurableObjectStorage};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    pub type DurableObjectState;

    #[wasm_bindgen(method, getter)]
    pub fn id(this: &DurableObjectState) -> DurableObjectId;

    #[wasm_bindgen(method, getter)]
    pub fn storage(this: &DurableObjectState) -> DurableObjectStorage;

    #[wasm_bindgen(method, js_name=waitUntil)]
    pub fn wait_until(this: &DurableObjectState, promise: &js_sys::Promise);
}
