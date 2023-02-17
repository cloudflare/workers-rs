use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object, js_name=DurableObjectTransaction)]
    pub type DurableObjectTransaction;

    #[wasm_bindgen(catch, method, js_class=DurableObjectTransaction, js_name=get)]
    pub fn get(this: &DurableObjectTransaction, key: &str) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class=DurableObjectTransaction, js_name=get)]
    pub fn get_multiple(
        this: &DurableObjectTransaction,
        keys: Vec<JsValue>,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class=DurableObjectTransaction, js_name=put)]
    pub fn put(
        this: &DurableObjectTransaction,
        key: &str,
        value: JsValue,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class=DurableObjectTransaction, js_name=put)]
    pub fn put_multiple(
        this: &DurableObjectTransaction,
        value: JsValue,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class=DurableObjectTransaction, js_name=delete)]
    pub fn delete(this: &DurableObjectTransaction, key: &str) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class=DurableObjectTransaction, js_name=delete)]
    pub fn delete_multiple(
        this: &DurableObjectTransaction,
        keys: Vec<JsValue>,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class=DurableObjectTransaction, js_name=deleteAll)]
    pub fn delete_all(this: &DurableObjectTransaction) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class=DurableObjectTransaction, js_name=list)]
    pub fn list(this: &DurableObjectTransaction) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class=DurableObjectTransaction, js_name=list)]
    pub fn list_with_options(
        this: &DurableObjectTransaction,
        options: js_sys::Object,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class=DurableObjectTransaction, js_name=rollback)]
    pub fn rollback(this: &DurableObjectTransaction) -> Result<(), JsValue>;
}
