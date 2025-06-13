use js_sys::Promise;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=Ai)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Ai;

    #[wasm_bindgen(structural, method, js_class=Ai, js_name=run)]
    pub fn run(this: &Ai, model: &str, input: JsValue) -> Promise;
}
