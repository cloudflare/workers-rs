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
    pub fn accept_web_socket(this: &DurableObjectState, ws: web_sys::WebSocket);

    #[wasm_bindgen(method, js_name=acceptWebSocket)]
    pub fn accept_web_socket_with_tags(
        this: &DurableObjectState,
        ws: web_sys::WebSocket,
        tags: Vec<String>,
    );

    #[wasm_bindgen(method, js_name=getWebSockets)]
    pub fn get_web_sockets(this: &DurableObjectState) -> Vec<web_sys::WebSocket>;

    #[wasm_bindgen(method, js_name=getWebSockets)]
    pub fn get_web_sockets_with_tag(
        this: &DurableObjectState,
        tag: String,
    ) -> Vec<web_sys::WebSocket>;
}
