use wasm_bindgen::prelude::*;

use crate::types::R2Object;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2Objects;

    #[wasm_bindgen(method, catch, getter)]
    pub fn objects(this: &R2Objects) -> Result<Vec<R2Object>, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn truncated(this: &R2Objects) -> Result<bool, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn cursor(this: &R2Objects) -> Result<Option<String>, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=delimitedPrefixes)]
    pub fn delimited_prefixes(this: &R2Objects) -> Result<Vec<js_sys::JsString>, JsValue>;
}
