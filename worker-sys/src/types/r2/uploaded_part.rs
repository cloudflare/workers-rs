use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object, js_name=R2UploadedPart)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2UploadedPart;

    #[wasm_bindgen(structural, method, getter, js_class=R2UploadedPart, js_name=partNumber)]
    pub fn part_number(this: &R2UploadedPart) -> u16;

    #[wasm_bindgen(structural, method, getter, js_class=R2UploadedPart, js_name=etag)]
    pub fn etag(this: &R2UploadedPart) -> String;
}
