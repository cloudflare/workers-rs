use js_sys::{Array, Promise};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=MessageBatch)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type MessageBatch;

    #[wasm_bindgen(method, getter,  js_class=MessageBatch, js_name=queue)]
    pub fn queue(this: &MessageBatch) -> String;

    #[wasm_bindgen(method, getter, js_class=MessageBatch, js_name=messages)]
    pub fn messages(this: &MessageBatch) -> Array;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen (extends = ::js_sys::Object, js_name = Queue)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Queue;

    #[wasm_bindgen(structural, method, js_class=Queue, js_name=send)]
    pub fn send(this: &Queue, mesage: JsValue) -> Promise;
}
