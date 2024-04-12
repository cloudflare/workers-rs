use ::js_sys::Object;
use wasm_bindgen::prelude::*;

use js_sys::{Array, Promise};

#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone)]
    pub type D1Result;

    #[wasm_bindgen(structural, method, catch, getter, js_name=results)]
    pub fn results(this: &D1Result) -> Result<Option<Array>, JsValue>;

    #[wasm_bindgen(structural, method, catch, getter, js_name=success)]
    pub fn success(this: &D1Result) -> Result<bool, JsValue>;

    #[wasm_bindgen(structural, method, catch, getter, js_name=error)]
    pub fn error(this: &D1Result) -> Result<Option<String>, JsValue>;

    #[wasm_bindgen(structural, method, catch, getter, js_name=meta)]
    pub fn meta(this: &D1Result) -> Result<Object, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone)]
    pub type D1ExecResult;

    #[wasm_bindgen(structural, method, catch, getter, js_name=count)]
    pub fn count(this: &D1ExecResult) -> Result<Option<u32>, JsValue>;

    #[wasm_bindgen(structural, method, catch, getter, js_name=duration)]
    pub fn duration(this: &D1ExecResult) -> Result<Option<f64>, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=D1Database)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type D1Database;

    #[wasm_bindgen(structural, method, catch, js_class=D1Database, js_name=prepare)]
    pub fn prepare(this: &D1Database, query: &str) -> Result<D1PreparedStatement, JsValue>;

    #[wasm_bindgen(structural, method, catch, js_class=D1Database, js_name=dump)]
    pub fn dump(this: &D1Database) -> Result<Promise, JsValue>;

    #[wasm_bindgen(structural, method, catch, js_class=D1Database, js_name=batch)]
    pub fn batch(this: &D1Database, statements: Array) -> Result<Promise, JsValue>;

    #[wasm_bindgen(structural, method, catch, js_class=D1Database, js_name=exec)]
    pub fn exec(this: &D1Database, query: &str) -> Result<Promise, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=D1PreparedStatement)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type D1PreparedStatement;

    #[wasm_bindgen(structural, method, catch, variadic, js_class=D1PreparedStatement, js_name=bind)]
    pub fn bind(this: &D1PreparedStatement, values: Array) -> Result<D1PreparedStatement, JsValue>;

    #[wasm_bindgen(structural, method, catch, js_class=D1PreparedStatement, js_name=first)]
    pub fn first(this: &D1PreparedStatement, col_name: Option<&str>) -> Result<Promise, JsValue>;

    #[wasm_bindgen(structural, method, catch, js_class=D1PreparedStatement, js_name=run)]
    pub fn run(this: &D1PreparedStatement) -> Result<Promise, JsValue>;

    #[wasm_bindgen(structural, method, catch, js_class=D1PreparedStatement, js_name=all)]
    pub fn all(this: &D1PreparedStatement) -> Result<Promise, JsValue>;

    #[wasm_bindgen(structural, method, catch, js_class=D1PreparedStatement, js_name=raw)]
    pub fn raw(this: &D1PreparedStatement) -> Result<Promise, JsValue>;
}
