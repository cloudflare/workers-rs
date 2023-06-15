use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type MessageBatch;

    #[wasm_bindgen(method, getter)]
    pub fn queue(this: &MessageBatch) -> js_sys::JsString;

    #[wasm_bindgen(method, getter)]
    pub fn messages(this: &MessageBatch) -> js_sys::Array;

    #[wasm_bindgen(method, js_name=retryAll)]
    pub fn retry_all(this: &MessageBatch);

    #[wasm_bindgen(method, js_name=ackAll)]
    pub fn ack_all(this: &MessageBatch);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Message;

    #[wasm_bindgen(method, getter)]
    pub fn id(this: &Message) -> js_sys::JsString;

    #[wasm_bindgen(method, getter)]
    pub fn timestamp(this: &Message) -> js_sys::Date;

    #[wasm_bindgen(method, getter)]
    pub fn body(this: &Message) -> JsValue;

    #[wasm_bindgen(method)]
    pub fn retry(this: &Message);

    #[wasm_bindgen(method)]
    pub fn ack(this: &Message);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Queue;

    #[wasm_bindgen(method)]
    pub fn send(this: &Queue, mesage: JsValue) -> js_sys::Promise;
}
