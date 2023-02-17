use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object, js_name=R2MultipartUpload)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2MultipartUpload;

    #[wasm_bindgen(structural, method, getter, js_class=R2MultipartUpload, js_name=key)]
    pub fn key(this: &R2MultipartUpload) -> String;

    #[wasm_bindgen(structural, method, getter, js_class=R2MultipartUpload, js_name=uploadId)]
    pub fn upload_id(this: &R2MultipartUpload) -> String;

    #[wasm_bindgen(structural, method, js_class=R2MultipartUpload, js_name=uploadPart)]
    pub fn upload_part(
        this: &R2MultipartUpload,
        part_number: u16,
        value: JsValue,
    ) -> js_sys::Promise;

    #[wasm_bindgen(structural, method, js_class=R2MultipartUpload, js_name=abort)]
    pub fn abort(this: &R2MultipartUpload) -> js_sys::Promise;

    #[wasm_bindgen(structural, method, js_class=R2MultipartUpload, js_name=complete)]
    pub fn complete(this: &R2MultipartUpload, uploaded_parts: Vec<JsValue>) -> js_sys::Promise;
}
