use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    #[derive(Clone)]
    #[wasm_bindgen(extends = js_sys::Object)]
    pub type SecretStoreSys;
    #[wasm_bindgen(method, catch, js_name = "get")]
    pub fn get(
        this: &SecretStoreSys,
    ) -> std::result::Result<js_sys::Promise, wasm_bindgen::JsValue>;
}
