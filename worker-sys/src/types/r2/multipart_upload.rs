use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2MultipartUpload;

    #[wasm_bindgen(method, catch, getter)]
    pub fn key(this: &R2MultipartUpload) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=uploadId)]
    pub fn upload_id(this: &R2MultipartUpload) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, js_name=uploadPart)]
    pub fn upload_part(
        this: &R2MultipartUpload,
        part_number: u16,
        value: JsValue,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn abort(this: &R2MultipartUpload) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn complete(
        this: &R2MultipartUpload,
        uploaded_parts: Vec<JsValue>,
    ) -> Result<js_sys::Promise, JsValue>;
}
