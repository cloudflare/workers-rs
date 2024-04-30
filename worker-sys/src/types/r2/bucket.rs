use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2Bucket;

    #[wasm_bindgen(method, catch)]
    pub fn head(this: &R2Bucket, key: String) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn get(this: &R2Bucket, key: String, options: JsValue) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn put(
        this: &R2Bucket,
        key: String,
        value: JsValue,
        options: JsValue,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn delete(this: &R2Bucket, key: String) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn list(this: &R2Bucket, options: JsValue) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name=createMultipartUpload)]
    pub fn create_multipart_upload(
        this: &R2Bucket,
        key: String,
        options: JsValue,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name=resumeMultipartUpload)]
    pub fn resume_multipart_upload(
        this: &R2Bucket,
        key: String,
        upload_id: String,
    ) -> Result<JsValue, JsValue>;
}
