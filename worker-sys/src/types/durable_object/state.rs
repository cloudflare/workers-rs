use wasm_bindgen::prelude::*;

use crate::types::durable_object::{DurableObjectId, DurableObjectStorage};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    pub type DurableObjectState;

    #[wasm_bindgen(method, getter)]
    pub fn id(this: &DurableObjectState) -> DurableObjectId;

    #[wasm_bindgen(method, getter)]
    pub fn storage(this: &DurableObjectState) -> DurableObjectStorage;
}
