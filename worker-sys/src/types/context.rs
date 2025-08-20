use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, PartialEq, Eq)]
    pub type Context;

    #[wasm_bindgen(method, catch, js_name=waitUntil)]
    pub fn wait_until(this: &Context, promise: &js_sys::Promise) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name=passThroughOnException)]
    pub fn pass_through_on_exception(this: &Context) -> Result<(), JsValue>;
}
