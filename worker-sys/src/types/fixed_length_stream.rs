use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=web_sys::TransformStream)]
    #[derive(Debug, Clone)]
    pub type FixedLengthStream;

    #[wasm_bindgen(constructor)]
    pub fn new(length: u32) -> FixedLengthStream;

    #[wasm_bindgen(constructor)]
    pub fn new_big_int(length: js_sys::BigInt) -> FixedLengthStream;

    #[wasm_bindgen(method, getter)]
    pub fn cron(this: &FixedLengthStream) -> String;
}
