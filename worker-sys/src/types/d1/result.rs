use ::js_sys::Object;
use wasm_bindgen::prelude::*;

use js_sys::Array;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type D1Result;

    #[wasm_bindgen(method, getter)]
    pub fn results(this: &D1Result) -> Option<Array>;

    #[wasm_bindgen(method, getter)]
    pub fn success(this: &D1Result) -> bool;

    #[wasm_bindgen(method, getter, js_name=error)]
    pub fn error(this: &D1Result) -> Option<String>;

    #[wasm_bindgen(method, getter, js_name=meta)]
    pub fn meta(this: &D1Result) -> Object;
}
