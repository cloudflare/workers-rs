use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen (extends = ::js_sys::Object , js_name = Headers , typescript_type = "Headers")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Headers;

    #[wasm_bindgen(catch, constructor, js_class = "Headers")]
    pub fn new() -> Result<Headers, JsValue>;

    #[wasm_bindgen(catch, constructor, js_class = "Headers")]
    pub fn new_with_headers(init: &Headers) -> Result<Headers, JsValue>;

    #[wasm_bindgen(catch, constructor, js_class = "Headers")]
    pub fn new_with_str_sequence_sequence(
        init: &::wasm_bindgen::JsValue,
    ) -> Result<Headers, JsValue>;

    #[wasm_bindgen (catch , method , structural , js_class = "Headers" , js_name = append)]
    pub fn append(this: &Headers, name: &str, value: &str) -> Result<(), JsValue>;

    #[wasm_bindgen (catch , method , structural , js_class = "Headers" , js_name = delete)]
    pub fn delete(this: &Headers, name: &str) -> Result<(), JsValue>;

    #[wasm_bindgen (catch , method , structural , js_class = "Headers" , js_name = get)]
    pub fn get(this: &Headers, name: &str) -> Result<Option<String>, JsValue>;

    #[wasm_bindgen (catch , method , structural , js_class = "Headers" , js_name = has)]
    pub fn has(this: &Headers, name: &str) -> Result<bool, JsValue>;

    #[wasm_bindgen (catch , method , structural , js_class = "Headers" , js_name = set)]
    pub fn set(this: &Headers, name: &str, value: &str) -> Result<(), JsValue>;

    #[wasm_bindgen (catch, method, structural, js_class = "Headers", js_name = entries)]
    pub fn entries(this: &Headers) -> Result<::js_sys::Iterator, JsValue>;

    #[wasm_bindgen (catch, method, structural, js_class = "Headers", js_name = keys)]
    pub fn keys(this: &Headers) -> Result<::js_sys::Iterator, JsValue>;

    #[wasm_bindgen (catch, method, structural, js_class = "Headers", js_name = values)]
    pub fn values(this: &Headers) -> Result<::js_sys::Iterator, JsValue>;
}
