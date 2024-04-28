use wasm_bindgen::prelude::*;

use crate::types::{DurableObjectId, DurableObjectStorage};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    pub type DurableObjectState;

    #[wasm_bindgen(method, catch, getter)]
    pub fn id(this: &DurableObjectState) -> Result<DurableObjectId, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn storage(this: &DurableObjectState) -> Result<DurableObjectStorage, JsValue>;

    #[wasm_bindgen(method, catch, js_name=waitUntil)]
    pub fn wait_until(this: &DurableObjectState, promise: &js_sys::Promise) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name=acceptWebSocket)]
    pub fn accept_websocket(
        this: &DurableObjectState,
        ws: &web_sys::WebSocket,
    ) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name=acceptWebSocket)]
    pub fn accept_websocket_with_tags(
        this: &DurableObjectState,
        ws: &web_sys::WebSocket,
        tags: Vec<JsValue>,
    ) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name=getWebSockets)]
    pub fn get_websockets(this: &DurableObjectState) -> Result<Vec<web_sys::WebSocket>, JsValue>;

    #[wasm_bindgen(method, catch, js_name=getWebSockets)]
    pub fn get_websockets_with_tag(
        this: &DurableObjectState,
        tag: &str,
    ) -> Result<Vec<web_sys::WebSocket>, JsValue>;

    #[wasm_bindgen(method, catch, js_name=getTags)]
    pub fn get_tags(
        this: &DurableObjectState,
        ws: &web_sys::WebSocket,
    ) -> Result<Vec<String>, JsValue>;
}
