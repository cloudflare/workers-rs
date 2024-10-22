use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Vectorize;

    #[wasm_bindgen(method, catch)]
    pub fn insert(this: &Vectorize, vectors: js_sys::Object) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn upsert(this: &Vectorize, vectors: js_sys::Object) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn describe(this: &Vectorize) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn query(
        this: &Vectorize,
        vector: JsValue,
        options: js_sys::Object,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name = "getByIds")]
    pub fn get_by_ids(this: &Vectorize, ids: JsValue) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name = "deleteByIds")]
    pub fn delete_by_ids(this: &Vectorize, ids: JsValue) -> Result<js_sys::Promise, JsValue>;
}
