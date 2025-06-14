use js_sys::JsString;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    pub type DurableObjectSqlStorage;

    #[wasm_bindgen(method, catch, js_name=exec, variadic)]
    pub fn exec(
        this: &DurableObjectSqlStorage,
        query: JsString,
        args: Vec<JsValue>,
    ) -> Result<DurableObjectSqlStorageCursor, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    pub type DurableObjectSqlStorageCursor;

    #[wasm_bindgen(method, catch, js_name=one)]
    pub fn one(this: &DurableObjectSqlStorageCursor) -> Result<JsValue, JsValue>;
}
