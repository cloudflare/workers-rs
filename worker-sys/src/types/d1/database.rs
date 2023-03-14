use wasm_bindgen::prelude::*;

use js_sys::{Array, Promise};

use crate::types::d1::prepared_statement::D1PreparedStatement;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object, js_name=D1Database)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type D1Database;

    #[wasm_bindgen(method, js_class=D1Database)]
    pub fn prepare(this: &D1Database, query: &str) -> D1PreparedStatement;

    #[wasm_bindgen(method, js_class=D1Database)]
    pub fn dump(this: &D1Database) -> Promise;

    #[wasm_bindgen(method, js_class=D1Database)]
    pub fn batch(this: &D1Database, statements: Array) -> Promise;

    #[wasm_bindgen(method, js_class=D1Database)]
    pub fn exec(this: &D1Database, query: &str) -> Promise;
}
