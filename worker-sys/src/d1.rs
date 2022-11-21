use ::js_sys::Object;
use wasm_bindgen::prelude::*;

use js_sys::{Array, JsString, Promise};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=D1Result)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type D1Result;

    #[wasm_bindgen(structural, method, getter, js_class=D1Result, js_name=results)]
    pub fn results(this: &D1Result) -> Array;

    #[wasm_bindgen(structural, method, getter, js_class=D1Result, js_name=success)]
    pub fn success(this: &D1Result) -> bool;

    #[wasm_bindgen(structural, method, getter, js_class=D1Result, js_name=error)]
    pub fn error(this: &D1Result) -> JsString;

    #[wasm_bindgen(structural, method, getter, js_class=D1Result, js_name=meta)]
    pub fn meta(this: &D1Result) -> Object;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=D1Database)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type D1Database;

    #[wasm_bindgen(structural, method, js_class=D1Database, js_name=prepare)]
    pub fn prepare(this: &D1Database, query: &str) -> D1PreparedStatement;

    #[wasm_bindgen(structural, method, js_class=D1Database, js_name=dump)]
    pub fn dump(this: &D1Database) -> Promise;

    #[wasm_bindgen(structural, method, js_class=D1Database, js_name=batch)]
    pub fn batch(this: &D1Database, statements: Array) -> Promise;

    #[wasm_bindgen(structural, method, js_class=D1Database, js_name=exec)]
    pub fn exec(this: &D1Database, query: &str) -> Promise;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=D1PreparedStatement)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type D1PreparedStatement;

    #[wasm_bindgen(structural, method, js_class=D1PreparedStatement, js_name=bind)]
    pub fn bind(this: &D1PreparedStatement, values: Array) -> D1PreparedStatement;

    #[wasm_bindgen(structural, method, js_class=D1PreparedStatement, js_name=first)]
    pub fn first(this: &D1PreparedStatement, col_name: Option<&str>) -> Promise;

    #[wasm_bindgen(structural, method, js_class=D1PreparedStatement, js_name=run)]
    pub fn run(this: &D1PreparedStatement) -> Promise;

    #[wasm_bindgen(structural, method, js_class=D1PreparedStatement, js_name=all)]
    pub fn all(this: &D1PreparedStatement) -> Promise;

    #[wasm_bindgen(structural, method, js_class=D1PreparedStatement, js_name=raw)]
    pub fn raw(this: &D1PreparedStatement) -> Promise;
}
