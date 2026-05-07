use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type WebSocketRequestResponsePair;

    #[wasm_bindgen(constructor, catch)]
    pub fn new(request: &str, response: &str) -> Result<WebSocketRequestResponsePair, JsValue>;

    #[wasm_bindgen(constructor, catch, js_class = "WebSocketRequestResponsePair")]
    pub fn new_bytes(
        request: &Uint8Array,
        response: &Uint8Array,
    ) -> Result<WebSocketRequestResponsePair, JsValue>;

    #[wasm_bindgen(method, getter)]
    pub fn request(this: &WebSocketRequestResponsePair) -> String;

    #[wasm_bindgen(method, getter)]
    pub fn response(this: &WebSocketRequestResponsePair) -> String;

    #[wasm_bindgen(method, getter, js_name = request)]
    pub fn request_bytes(this: &WebSocketRequestResponsePair) -> Uint8Array;

    #[wasm_bindgen(method, getter, js_name = response)]
    pub fn response_bytes(this: &WebSocketRequestResponsePair) -> Uint8Array;
}
