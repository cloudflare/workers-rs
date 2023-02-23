use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2UploadedPart;

    #[wasm_bindgen(method, getter, js_name=partNumber)]
    pub fn part_number(this: &R2UploadedPart) -> u16;

    #[wasm_bindgen(method, getter)]
    pub fn etag(this: &R2UploadedPart) -> String;
}
