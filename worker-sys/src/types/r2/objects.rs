use wasm_bindgen::prelude::*;

use crate::types::R2Object;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object, js_name=R2Objects)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2Objects;

    #[wasm_bindgen(structural, method, getter, js_class=R2Objects, js_name=objects)]
    pub fn objects(this: &R2Objects) -> Vec<R2Object>;

    #[wasm_bindgen(structural, method, getter, js_class=R2Objects, js_name=truncated)]
    pub fn truncated(this: &R2Objects) -> bool;

    #[wasm_bindgen(structural, method, getter, js_class=R2Objects, js_name=cursor)]
    pub fn cursor(this: &R2Objects) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=R2Objects, js_name=delimitedPrefixes)]
    pub fn delimited_prefixes(this: &R2Objects) -> Vec<js_sys::JsString>;
}
