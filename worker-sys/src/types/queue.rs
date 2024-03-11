use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type MessageBatch;

    #[wasm_bindgen(method, getter)]
    pub fn queue(this: &MessageBatch) -> String;

    #[wasm_bindgen(method, getter)]
    pub fn messages(this: &MessageBatch) -> js_sys::Array;

    #[wasm_bindgen(method, js_name=retryAll)]
    pub fn retry_all(this: &MessageBatch);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Queue;

    #[wasm_bindgen(method)]
    pub fn send(this: &Queue, message: JsValue) -> js_sys::Promise;
}
