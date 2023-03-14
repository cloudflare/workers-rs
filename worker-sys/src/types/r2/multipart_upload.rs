use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2MultipartUpload;

    #[wasm_bindgen(method, getter)]
    pub fn key(this: &R2MultipartUpload) -> String;

    #[wasm_bindgen(method, getter, js_name=uploadId)]
    pub fn upload_id(this: &R2MultipartUpload) -> String;

    #[wasm_bindgen(method, js_name=uploadPart)]
    pub fn upload_part(
        this: &R2MultipartUpload,
        part_number: u16,
        value: JsValue,
    ) -> js_sys::Promise;

    #[wasm_bindgen(method)]
    pub fn abort(this: &R2MultipartUpload) -> js_sys::Promise;

    #[wasm_bindgen(method)]
    pub fn complete(this: &R2MultipartUpload, uploaded_parts: Vec<JsValue>) -> js_sys::Promise;
}
