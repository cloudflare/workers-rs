use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = js_sys::Object)]
    #[derive(Clone, Debug)]
    pub type SyncKvStorage;

    #[wasm_bindgen(method)]
    pub fn get(this: &SyncKvStorage, key: &str) -> JsValue;

    #[wasm_bindgen(method)]
    pub fn put(this: &SyncKvStorage, key: &str, value: JsValue);

    #[wasm_bindgen(method)]
    pub fn delete(this: &SyncKvStorage, key: &str) -> bool;

    #[wasm_bindgen(method)]
    pub fn list(this: &SyncKvStorage) -> js_sys::Object;

    #[wasm_bindgen(method, js_name = list)]
    pub fn list_with_options(this: &SyncKvStorage, options: js_sys::Object) -> js_sys::Object;
}
