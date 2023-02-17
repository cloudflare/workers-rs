use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object, js_name=R2Bucket)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2Bucket;

    #[wasm_bindgen(structural, method, js_class=R2Bucket, js_name=head)]
    pub fn head(this: &R2Bucket, key: String) -> js_sys::Promise;

    #[wasm_bindgen(structural, method, js_class=R2Bucket, js_name=get)]
    pub fn get(this: &R2Bucket, key: String, options: JsValue) -> js_sys::Promise;

    #[wasm_bindgen(structural, method, js_class=R2Bucket, js_name=put)]
    pub fn put(this: &R2Bucket, key: String, value: JsValue, options: JsValue) -> js_sys::Promise;

    #[wasm_bindgen(structural, method, js_class=R2Bucket, js_name=delete)]
    pub fn delete(this: &R2Bucket, key: String) -> js_sys::Promise;

    #[wasm_bindgen(structural, method, js_class=R2Bucket, js_name=list)]
    pub fn list(this: &R2Bucket, options: JsValue) -> js_sys::Promise;

    #[wasm_bindgen(structural, method, js_class=R2Bucket, js_name=createMultipartUpload)]
    pub fn create_multipart_upload(
        this: &R2Bucket,
        key: String,
        options: JsValue,
    ) -> js_sys::Promise;

    #[wasm_bindgen(structural, method, js_class=R2Bucket, js_name=resumeMultipartUpload)]
    pub fn resume_multipart_upload(this: &R2Bucket, key: String, upload_id: String) -> JsValue;
}
