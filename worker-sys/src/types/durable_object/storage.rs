use wasm_bindgen::prelude::*;

use crate::types::DurableObjectTransaction;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    pub type DurableObjectStorage;

    #[wasm_bindgen(method, catch)]
    pub fn get(this: &DurableObjectStorage, key: &str) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name=get)]
    pub fn get_multiple(
        this: &DurableObjectStorage,
        keys: Vec<JsValue>,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn put(
        this: &DurableObjectStorage,
        key: &str,
        value: JsValue,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name=put)]
    pub fn put_multiple(
        this: &DurableObjectStorage,
        value: JsValue,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn delete(this: &DurableObjectStorage, key: &str) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name=delete)]
    pub fn delete_multiple(
        this: &DurableObjectStorage,
        keys: Vec<JsValue>,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name=deleteAll)]
    pub fn delete_all(this: &DurableObjectStorage) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn list(this: &DurableObjectStorage) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name=list)]
    pub fn list_with_options(
        this: &DurableObjectStorage,
        options: js_sys::Object,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn transaction(
        this: &DurableObjectStorage,
        closure: &Closure<dyn FnMut(DurableObjectTransaction) -> js_sys::Promise>,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name=getAlarm)]
    pub fn get_alarm(
        this: &DurableObjectStorage,
        options: js_sys::Object,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name=setAlarm)]
    pub fn set_alarm(
        this: &DurableObjectStorage,
        scheduled_time: js_sys::Date,
        options: js_sys::Object,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name=deleteAlarm)]
    pub fn delete_alarm(
        this: &DurableObjectStorage,
        options: js_sys::Object,
    ) -> Result<js_sys::Promise, JsValue>;
}
