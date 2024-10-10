use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2UploadedPart;

    #[wasm_bindgen(method, catch, getter, js_name=partNumber)]
    pub fn part_number(this: &R2UploadedPart) -> Result<u16, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn etag(this: &R2UploadedPart) -> Result<String, JsValue>;
}
