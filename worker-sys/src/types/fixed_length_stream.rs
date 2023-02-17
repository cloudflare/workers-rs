use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=web_sys::TransformStream, js_name=FixedLengthStream)]
    #[derive(Debug, Clone)]
    pub type FixedLengthStream;

    #[wasm_bindgen(constructor, js_class=FixedLengthStream)]
    pub fn new(length: u32) -> FixedLengthStream;

    #[wasm_bindgen(constructor, js_class=FixedLengthStream)]
    pub fn new_big_int(length: js_sys::BigInt) -> FixedLengthStream;

    #[wasm_bindgen(structural, method, getter, js_class=FixedLengthStream, js_name=cron)]
    pub fn cron(this: &FixedLengthStream) -> String;
}
