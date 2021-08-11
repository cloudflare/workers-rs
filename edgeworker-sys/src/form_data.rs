use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=FormData)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type FormData;

    #[wasm_bindgen(structural, method, js_class=FormData, js_name=append)]
    pub fn append(
        this: &FormData,
        name: String,
        value: String,
        // ok so the deal is this can be undefined, actually
        filename: JsValue,
    );

    #[wasm_bindgen(structural, method, js_class=FormData, js_name=delete)]
    pub fn delete(this: &FormData, name: String);

    #[wasm_bindgen(structural, method, js_class=FormData, js_name=entries)]
    pub fn entries(this: &FormData) -> js_sys::Iterator;

    #[wasm_bindgen(structural, method, js_class=FormData, js_name=get)]
    pub fn get(this: &FormData, name: String) -> String;

    #[wasm_bindgen(structural, method, js_class=FormData, js_name=getAll)]
    pub fn get_all(this: &FormData, name: String) -> js_sys::Array;

    #[wasm_bindgen(structural, method, js_class=FormData, js_name=has)]
    pub fn has(this: &FormData, name: String) -> bool;

    #[wasm_bindgen(structural, method, js_class=FormData, js_name=keys)]
    pub fn keys(this: &FormData) -> js_sys::Iterator;

    #[wasm_bindgen(structural, method, js_class=FormData, js_name=set)]
    pub fn set(
        this: &FormData,
        name: String,
        value: String,
        filename: JsValue, // possibly undefined
    );

    #[wasm_bindgen(structural, method, js_class=FormData, js_name=values)]
    pub fn values(this: &FormData) -> js_sys::Iterator;

}
