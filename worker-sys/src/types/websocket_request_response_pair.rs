use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type WebSocketRequestResponsePair;

    #[wasm_bindgen(constructor, catch)]
    pub fn new(request: &str, response: &str) -> Result<WebSocketRequestResponsePair, JsValue>;

    #[wasm_bindgen(method, getter)]
    pub fn request(this: &WebSocketRequestResponsePair) -> String;

    #[wasm_bindgen(method, getter)]
    pub fn response(this: &WebSocketRequestResponsePair) -> String;
}
