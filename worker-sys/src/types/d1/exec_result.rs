use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone)]
    pub type D1ExecResult;

    #[wasm_bindgen(method, getter)]
    pub fn count(this: &D1ExecResult) -> Option<u32>;

    #[wasm_bindgen(method, getter)]
    pub fn duration(this: &D1ExecResult) -> Option<f64>;
}
