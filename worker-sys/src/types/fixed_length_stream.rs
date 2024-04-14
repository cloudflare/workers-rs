use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=web_sys::TransformStream)]
    #[derive(Debug, Clone)]
    pub type FixedLengthStream;

    #[wasm_bindgen(constructor, catch)]
    pub fn new(length: u32) -> Result<FixedLengthStream, JsValue>;

    #[wasm_bindgen(constructor, catch)]
    pub fn new_big_int(length: js_sys::BigInt) -> Result<FixedLengthStream, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn cron(this: &FixedLengthStream) -> Result<String, JsValue>;
}
