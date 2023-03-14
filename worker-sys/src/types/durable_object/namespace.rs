use wasm_bindgen::prelude::*;

use crate::types::{DurableObject, DurableObjectId};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    pub type DurableObjectNamespace;

    #[wasm_bindgen(method, catch, js_name=idFromName)]
    pub fn id_from_name(
        this: &DurableObjectNamespace,
        name: &str,
    ) -> Result<DurableObjectId, JsValue>;

    #[wasm_bindgen(method, catch, js_name=idFromString)]
    pub fn id_from_string(
        this: &DurableObjectNamespace,
        string: &str,
    ) -> Result<DurableObjectId, JsValue>;

    #[wasm_bindgen(method, catch, js_name=newUniqueId)]
    pub fn new_unique_id(this: &DurableObjectNamespace) -> Result<DurableObjectId, JsValue>;

    #[wasm_bindgen(method, catch, js_name=newUniqueId)]
    pub fn new_unique_id_with_options(
        this: &DurableObjectNamespace,
        options: &JsValue,
    ) -> Result<DurableObjectId, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn get(
        this: &DurableObjectNamespace,
        id: &DurableObjectId,
    ) -> Result<DurableObject, JsValue>;
}
