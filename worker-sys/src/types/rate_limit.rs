use js_sys::Promise;
use wasm_bindgen::JsValue;

#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Clone, PartialEq, Eq)]
    pub type RateLimiter;

    #[wasm_bindgen(method, catch)]
    pub fn limit(this: &RateLimiter, arg: js_sys::Object) -> Result<Promise, JsValue>;
}
