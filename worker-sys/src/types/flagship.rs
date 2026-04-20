use js_sys::Promise;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object, js_name=Flagship)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Flagship;

    #[wasm_bindgen(method, js_class=Flagship, js_name=get)]
    pub fn get(
        this: &Flagship,
        flag_key: &str,
        default_value: JsValue,
        context: JsValue,
    ) -> Promise;

    #[wasm_bindgen(method, js_class=Flagship, js_name=getBooleanValue)]
    pub fn get_boolean_value(
        this: &Flagship,
        flag_key: &str,
        default_value: bool,
        context: JsValue,
    ) -> Promise;

    #[wasm_bindgen(method, js_class=Flagship, js_name=getStringValue)]
    pub fn get_string_value(
        this: &Flagship,
        flag_key: &str,
        default_value: &str,
        context: JsValue,
    ) -> Promise;

    #[wasm_bindgen(method, js_class=Flagship, js_name=getNumberValue)]
    pub fn get_number_value(
        this: &Flagship,
        flag_key: &str,
        default_value: f64,
        context: JsValue,
    ) -> Promise;

    #[wasm_bindgen(method, js_class=Flagship, js_name=getObjectValue)]
    pub fn get_object_value(
        this: &Flagship,
        flag_key: &str,
        default_value: JsValue,
        context: JsValue,
    ) -> Promise;

    #[wasm_bindgen(method, js_class=Flagship, js_name=getBooleanDetails)]
    pub fn get_boolean_details(
        this: &Flagship,
        flag_key: &str,
        default_value: bool,
        context: JsValue,
    ) -> Promise;

    #[wasm_bindgen(method, js_class=Flagship, js_name=getStringDetails)]
    pub fn get_string_details(
        this: &Flagship,
        flag_key: &str,
        default_value: &str,
        context: JsValue,
    ) -> Promise;

    #[wasm_bindgen(method, js_class=Flagship, js_name=getNumberDetails)]
    pub fn get_number_details(
        this: &Flagship,
        flag_key: &str,
        default_value: f64,
        context: JsValue,
    ) -> Promise;

    #[wasm_bindgen(method, js_class=Flagship, js_name=getObjectDetails)]
    pub fn get_object_details(
        this: &Flagship,
        flag_key: &str,
        default_value: JsValue,
        context: JsValue,
    ) -> Promise;
}
