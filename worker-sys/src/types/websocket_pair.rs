use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object, js_name=WebSocketPair)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    /// The `WebSocketPair` dictionary.
    pub type WebSocketPair;

    #[wasm_bindgen(constructor, js_class=WebSocketPair)]
    pub fn new() -> WebSocketPair;
}

impl WebSocketPair {
    pub fn client(&mut self) -> Result<web_sys::WebSocket, JsValue> {
        let value = js_sys::Reflect::get(self.as_ref(), &JsValue::from("0"))?;
        Ok(web_sys::WebSocket::from(value))
    }

    pub fn server(&mut self) -> Result<web_sys::WebSocket, JsValue> {
        let value = js_sys::Reflect::get(self.as_ref(), &JsValue::from("1"))?;
        Ok(web_sys::WebSocket::from(value))
    }
}
