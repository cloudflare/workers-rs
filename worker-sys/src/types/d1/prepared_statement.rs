use wasm_bindgen::prelude::*;

use js_sys::{Array, Promise};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object, js_name=D1PreparedStatement)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type D1PreparedStatement;

    #[wasm_bindgen(method, catch, variadic, js_class=D1PreparedStatement)]
    pub fn bind(this: &D1PreparedStatement, values: Array) -> Result<D1PreparedStatement, JsValue>;

    #[wasm_bindgen(method, js_class=D1PreparedStatement)]
    pub fn first(this: &D1PreparedStatement, col_name: Option<&str>) -> Promise;

    #[wasm_bindgen(method, js_class=D1PreparedStatement)]
    pub fn run(this: &D1PreparedStatement) -> Promise;

    #[wasm_bindgen(method, js_class=D1PreparedStatement)]
    pub fn all(this: &D1PreparedStatement) -> Promise;

    #[wasm_bindgen(method, js_class=D1PreparedStatement)]
    pub fn raw(this: &D1PreparedStatement) -> Promise;
}
