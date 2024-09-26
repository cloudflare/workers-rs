use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type VectorizeIndex;

    #[wasm_bindgen(method, catch)]
    pub fn insert(
        this: &VectorizeIndex,
        vectors: js_sys::Object,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn upsert(
        this: &VectorizeIndex,
        vectors: js_sys::Object,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn describe(this: &VectorizeIndex) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn query(
        this: &VectorizeIndex,
        vector: &[f32],
        options: js_sys::Object,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name = "getByIds")]
    pub fn get_by_ids(this: &VectorizeIndex, ids: JsValue) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name = "deleteByIds")]
    pub fn delete_by_ids(this: &VectorizeIndex, ids: JsValue) -> Result<js_sys::Promise, JsValue>;
}
