use wasm_bindgen::prelude::*;

use crate::types::R2Object;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2Objects;

    #[wasm_bindgen(method, getter)]
    pub fn objects(this: &R2Objects) -> Vec<R2Object>;

    #[wasm_bindgen(method, getter)]
    pub fn truncated(this: &R2Objects) -> bool;

    #[wasm_bindgen(method, getter)]
    pub fn cursor(this: &R2Objects) -> Option<String>;

    #[wasm_bindgen(method, getter, js_name=delimitedPrefixes)]
    pub fn delimited_prefixes(this: &R2Objects) -> Vec<js_sys::JsString>;
}
