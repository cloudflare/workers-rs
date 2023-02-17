use wasm_bindgen::prelude::*;

use crate::types::{DurableObjectId, DurableObjectStorage};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object, js_name=DurableObjectState)]
    pub type DurableObjectState;

    #[wasm_bindgen(method, getter, js_class=DurableObjectState, js_name=id)]
    pub fn id(this: &DurableObjectState) -> DurableObjectId;

    #[wasm_bindgen(method, getter, js_class=DurableObjectState, js_name=storage)]
    pub fn storage(this: &DurableObjectState) -> DurableObjectStorage;
}
