use js_sys::Promise;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type Images;

    #[wasm_bindgen(method, js_name = fetch)]
    pub fn fetch(this: &Images, input: JsValue, options: JsValue) -> Promise;
}
