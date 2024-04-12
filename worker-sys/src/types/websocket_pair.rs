use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    /// The `WebSocketPair` dictionary.
    pub type WebSocketPair;

    #[wasm_bindgen(constructor, catch)]
    pub fn new() -> Result<WebSocketPair, JsValue>;
}

impl WebSocketPair {
    pub fn client(&mut self) -> Result<web_sys::WebSocket, JsValue> {
        let value = js_sys::Reflect::get_u32(self.as_ref(), 0)?;
        Ok(web_sys::WebSocket::from(value))
    }

    pub fn server(&mut self) -> Result<web_sys::WebSocket, JsValue> {
        let value = js_sys::Reflect::get_u32(self.as_ref(), 1)?;
        Ok(web_sys::WebSocket::from(value))
    }
}
