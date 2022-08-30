use wasm_bindgen::prelude::*;

use crate::fetcher::Fetcher;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = ::js_sys::Object, js_name = DynamicDispatcher)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type DynamicDispatcher;

    #[wasm_bindgen(structural, method, js_class=DynamicDispatcher, js_name = get, catch)]
    pub fn get(this: &DynamicDispatcher, name: String, options: JsValue) -> Result<Fetcher, JsValue>;
}
