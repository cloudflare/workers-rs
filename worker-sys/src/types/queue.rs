use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type MessageBatch;

    #[wasm_bindgen(method, catch, getter)]
    pub fn queue(this: &MessageBatch) -> Result<js_sys::JsString, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn messages(this: &MessageBatch) -> Result<js_sys::Array, JsValue>;

    #[wasm_bindgen(method, catch, js_name=retryAll)]
    pub fn retry_all(this: &MessageBatch, options: JsValue) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name=ackAll)]
    pub fn ack_all(this: &MessageBatch) -> Result<(), JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Message;

    #[wasm_bindgen(method, catch, getter)]
    pub fn id(this: &Message) -> Result<js_sys::JsString, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn timestamp(this: &Message) -> Result<js_sys::Date, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn body(this: &Message) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, getter)]
    pub fn attempts(this: &Message) -> u32;

    #[wasm_bindgen(method, catch)]
    pub fn retry(this: &Message, options: JsValue) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn ack(this: &Message) -> Result<(), JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Queue;

    #[wasm_bindgen(method, catch)]
    pub fn send(
        this: &Queue,
        message: JsValue,
        options: JsValue,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name=sendBatch)]
    pub fn send_batch(
        this: &Queue,
        messages: js_sys::Array,
        options: JsValue,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn metrics(this: &Queue) -> Result<js_sys::Promise, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type QueueMetrics;

    #[wasm_bindgen(method, getter, js_name=backlogCount)]
    pub fn backlog_count(this: &QueueMetrics) -> f64;

    #[wasm_bindgen(method, getter, js_name=backlogBytes)]
    pub fn backlog_bytes(this: &QueueMetrics) -> f64;

    #[wasm_bindgen(method, getter, js_name=oldestMessageTimestamp)]
    pub fn oldest_message_timestamp(this: &QueueMetrics) -> Option<js_sys::Date>;
}
