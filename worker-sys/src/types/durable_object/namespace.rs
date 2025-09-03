use wasm_bindgen::prelude::*;

use crate::types::{DurableObject, DurableObjectId};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone)]
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

    #[wasm_bindgen(method, catch, js_name=get)]
    pub fn get_with_options(
        this: &DurableObjectNamespace,
        id: &DurableObjectId,
        options: &JsValue,
    ) -> Result<DurableObject, JsValue>;

    #[wasm_bindgen(method, catch, js_name=getByName)]
    pub fn get_by_name(this: &DurableObjectNamespace, name: &str)
        -> Result<DurableObject, JsValue>;

    #[wasm_bindgen(method, catch, js_name=getByName)]
    pub fn get_by_name_with_options(
        this: &DurableObjectNamespace,
        name: &str,
        options: &JsValue,
    ) -> Result<DurableObject, JsValue>;
}
