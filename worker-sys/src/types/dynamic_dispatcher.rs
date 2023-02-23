use wasm_bindgen::prelude::*;

use crate::types::Fetcher;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type DynamicDispatcher;

    #[wasm_bindgen(method, catch)]
    pub fn get(
        this: &DynamicDispatcher,
        name: String,
        options: JsValue,
    ) -> Result<Fetcher, JsValue>;
}
