use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2Bucket;

    #[wasm_bindgen(method)]
    pub fn head(this: &R2Bucket, key: String) -> js_sys::Promise;

    #[wasm_bindgen(method)]
    pub fn get(this: &R2Bucket, key: String, options: JsValue) -> js_sys::Promise;

    #[wasm_bindgen(method)]
    pub fn put(this: &R2Bucket, key: String, value: JsValue, options: JsValue) -> js_sys::Promise;

    #[wasm_bindgen(method)]
    pub fn delete(this: &R2Bucket, key: String) -> js_sys::Promise;

    #[wasm_bindgen(method)]
    pub fn list(this: &R2Bucket, options: JsValue) -> js_sys::Promise;

    #[wasm_bindgen(method, js_name=createMultipartUpload)]
    pub fn create_multipart_upload(
        this: &R2Bucket,
        key: String,
        options: JsValue,
    ) -> js_sys::Promise;

    #[wasm_bindgen(method, js_name=resumeMultipartUpload)]
    pub fn resume_multipart_upload(this: &R2Bucket, key: String, upload_id: String) -> JsValue;
}
