use wasm_bindgen::prelude::*;

use crate::types::{DurableObjectId, DurableObjectStorage};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    pub type DurableObjectState;

    #[wasm_bindgen(method, getter)]
    pub fn id(this: &DurableObjectState) -> DurableObjectId;

    #[wasm_bindgen(method, getter)]
    pub fn storage(this: &DurableObjectState) -> DurableObjectStorage;

    #[wasm_bindgen(method, js_name=waitUntil)]
    pub fn wait_until(this: &DurableObjectState, promise: &js_sys::Promise);

    #[wasm_bindgen(method, js_name=acceptWebSocket)]
    pub fn accept_websocket(this: &DurableObjectState, ws: &web_sys::WebSocket);

    #[wasm_bindgen(method, js_name=acceptWebSocket)]
    pub fn accept_websocket_with_tags(
        this: &DurableObjectState,
        ws: &web_sys::WebSocket,
        tags: Vec<JsValue>,
    );

    #[wasm_bindgen(method, js_name=getWebSockets)]
    pub fn get_websockets(this: &DurableObjectState) -> Vec<web_sys::WebSocket>;

    #[wasm_bindgen(method, js_name=getWebSockets)]
    pub fn get_websockets_with_tag(this: &DurableObjectState, tag: &str)
        -> Vec<web_sys::WebSocket>;

    #[wasm_bindgen(method, js_name=getTags)]
    pub fn get_tags(this: &DurableObjectState, ws: &web_sys::WebSocket) -> Vec<String>;
}
