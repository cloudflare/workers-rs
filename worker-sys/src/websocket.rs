use crate::global::EventTarget;
use js_sys::Uint8Array;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {

    #[wasm_bindgen(extends = js_sys::Object, js_name = WebSocketPair)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `WebSocketPair` dictionary."]
    pub type WebSocketPair;

    #[wasm_bindgen(constructor, js_class = WebSocketPair)]
    pub fn new() -> WebSocketPair;
}

impl WebSocketPair {
    pub fn client(&mut self) -> Result<WebSocket, JsValue> {
        let value = ::js_sys::Reflect::get(self.as_ref(), &JsValue::from("0"))?;
        Ok(WebSocket::from(value))
    }

    pub fn server(&mut self) -> Result<WebSocket, JsValue> {
        let value = ::js_sys::Reflect::get(self.as_ref(), &JsValue::from("1"))?;
        Ok(WebSocket::from(value))
    }
}

#[wasm_bindgen]
extern "C" {

    #[wasm_bindgen(extends = EventTarget, extends = js_sys::Object, js_name = WebSocket)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `WebSocket` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)"]
    pub type WebSocket;

    #[wasm_bindgen(catch, structural, method, js_class = "WebSocket", js_name = accept)]
    #[doc = "Accepts the server side of the WebSocket."]
    #[doc = ""]
    #[doc = "[CF Documentation](https://developers.cloudflare.com/workers/runtime-apis/websockets#accept)"]
    pub fn accept(this: &WebSocket) -> Result<(), JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class = "WebSocket", js_name = close)]
    #[doc = "The `close()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close)"]
    pub fn close(this: &WebSocket) -> Result<(), JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class = "WebSocket", js_name = close)]
    #[doc = "The `close()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close)"]
    pub fn close_with_code(this: &WebSocket, code: u16) -> Result<(), JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class = "WebSocket", js_name = close)]
    #[doc = "The `close()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close)"]
    pub fn close_with_code_and_reason(
        this: &WebSocket,
        code: u16,
        reason: &str,
    ) -> Result<(), JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class = "WebSocket", js_name = send)]
    #[doc = "The `send()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/send)"]
    pub fn send_with_str(this: &WebSocket, data: &str) -> Result<(), JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class = "WebSocket", js_name = send)]
    #[doc = "The `send()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/send)"]
    pub fn send_with_u8_array(this: &WebSocket, data: Uint8Array) -> Result<(), JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class = "WebSocket", js_name = addEventListener)]
    #[doc = "The `addEventListener()` method."]
    #[doc = ""]
    #[doc = "[CF Documentation](https://developers.cloudflare.com/workers/runtime-apis/websockets#addeventlistener)"]
    pub fn add_event_listener(
        this: &WebSocket,
        r#type: JsValue,
        value: Option<&::js_sys::Function>,
    ) -> Result<(), JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class = "WebSocket", js_name = removeEventListener)]
    #[doc = "The `removeEventListener()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](hhttps://developer.mozilla.org/en-US/docs/Web/API/EventTarget/removeEventListener)"]
    pub fn remove_event_listener(
        this: &WebSocket,
        r#type: JsValue,
        value: Option<&::js_sys::Function>,
    ) -> Result<(), JsValue>;
}
