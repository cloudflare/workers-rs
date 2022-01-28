use crate::global::EventTarget;
use js_sys::Uint8Array;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen (extends = :: js_sys :: Object , js_name = WebSocketPair)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `WebSocketPair` dictionary."]
    pub type WebSocketPair;

    #[wasm_bindgen(constructor, js_class=WebSocketPair)]
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
    # [wasm_bindgen (extends = EventTarget , extends = :: js_sys :: Object , js_name = WebSocket , typescript_type = "WebSocket")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `WebSocket` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)"]
    pub type WebSocket;
    # [wasm_bindgen (catch, structural, method, js_class = "WebSocket", js_name = accept)]
    #[doc = "Accepts the server side of the WebSocket."]
    #[doc = ""]
    #[doc = "[CF Documentation](https://developers.cloudflare.com/workers/runtime-apis/websockets#accept)"]
    pub fn accept(this: &WebSocket) -> Result<(), JsValue>;
    # [wasm_bindgen (structural , method , getter , js_class = "WebSocket" , js_name = url)]
    #[doc = "Getter for the `url` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/url)"]
    pub fn url(this: &WebSocket) -> String;
    # [wasm_bindgen (structural , method , getter , js_class = "WebSocket" , js_name = readyState)]
    #[doc = "Getter for the `readyState` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/readyState)"]
    pub fn ready_state(this: &WebSocket) -> u16;
    # [wasm_bindgen (structural , method , getter , js_class = "WebSocket" , js_name = bufferedAmount)]
    #[doc = "Getter for the `bufferedAmount` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/bufferedAmount)"]
    pub fn buffered_amount(this: &WebSocket) -> u32;
    # [wasm_bindgen (structural , method , getter , js_class = "WebSocket" , js_name = onopen)]
    #[doc = "Getter for the `onopen` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/onopen)"]
    pub fn onopen(this: &WebSocket) -> Option<::js_sys::Function>;
    # [wasm_bindgen (structural , method , setter , js_class = "WebSocket" , js_name = onopen)]
    #[doc = "Setter for the `onopen` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/onopen)"]
    pub fn set_onopen(this: &WebSocket, value: Option<&::js_sys::Function>);
    # [wasm_bindgen (structural , method , getter , js_class = "WebSocket" , js_name = onerror)]
    #[doc = "Getter for the `onerror` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/onerror)"]
    pub fn onerror(this: &WebSocket) -> Option<::js_sys::Function>;
    # [wasm_bindgen (structural , method , setter , js_class = "WebSocket" , js_name = onerror)]
    #[doc = "Setter for the `onerror` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/onerror)"]
    pub fn set_onerror(this: &WebSocket, value: Option<&::js_sys::Function>);
    # [wasm_bindgen (structural , method , getter , js_class = "WebSocket" , js_name = onclose)]
    #[doc = "Getter for the `onclose` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/onclose)"]
    pub fn onclose(this: &WebSocket) -> Option<::js_sys::Function>;
    # [wasm_bindgen (structural , method , setter , js_class = "WebSocket" , js_name = onclose)]
    #[doc = "Setter for the `onclose` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/onclose)"]
    pub fn set_onclose(this: &WebSocket, value: Option<&::js_sys::Function>);
    # [wasm_bindgen (structural , method , getter , js_class = "WebSocket" , js_name = extensions)]
    #[doc = "Getter for the `extensions` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/extensions)"]
    pub fn extensions(this: &WebSocket) -> String;
    # [wasm_bindgen (structural , method , getter , js_class = "WebSocket" , js_name = protocol)]
    #[doc = "Getter for the `protocol` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/protocol)"]
    pub fn protocol(this: &WebSocket) -> String;
    # [wasm_bindgen (structural , method , getter , js_class = "WebSocket" , js_name = onmessage)]
    #[doc = "Getter for the `onmessage` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/onmessage)"]
    pub fn onmessage(this: &WebSocket) -> Option<::js_sys::Function>;
    # [wasm_bindgen (structural , method , setter , js_class = "WebSocket" , js_name = onmessage)]
    #[doc = "Setter for the `onmessage` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/onmessage)"]
    pub fn set_onmessage(this: &WebSocket, value: Option<&::js_sys::Function>);
    # [wasm_bindgen (structural , method , getter , js_class = "WebSocket" , js_name = binaryType)]
    #[doc = "Getter for the `binaryType` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/binaryType)"]
    pub fn binary_type(this: &WebSocket) -> web_sys::BinaryType;
    # [wasm_bindgen (structural , method , setter , js_class = "WebSocket" , js_name = binaryType)]
    #[doc = "Setter for the `binaryType` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/binaryType)"]
    pub fn set_binary_type(this: &WebSocket, value: web_sys::BinaryType);
    #[wasm_bindgen(catch, constructor, js_class = "WebSocket")]
    #[doc = "The `new WebSocket(..)` constructor, creating a new instance of `WebSocket`."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/WebSocket)"]
    pub fn new(url: &str) -> Result<WebSocket, JsValue>;
    #[wasm_bindgen(catch, constructor, js_class = "WebSocket")]
    #[doc = "The `new WebSocket(..)` constructor, creating a new instance of `WebSocket`."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/WebSocket)"]
    pub fn new_with_str(url: &str, protocols: &str) -> Result<WebSocket, JsValue>;
    #[wasm_bindgen(catch, constructor, js_class = "WebSocket")]
    #[doc = "The `new WebSocket(..)` constructor, creating a new instance of `WebSocket`."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/WebSocket)"]
    pub fn new_with_str_sequence(
        url: &str,
        protocols: &::wasm_bindgen::JsValue,
    ) -> Result<WebSocket, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WebSocket" , js_name = close)]
    #[doc = "The `close()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close)"]
    pub fn close(this: &WebSocket) -> Result<(), JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WebSocket" , js_name = close)]
    #[doc = "The `close()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close)"]
    pub fn close_with_code(this: &WebSocket, code: u16) -> Result<(), JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WebSocket" , js_name = close)]
    #[doc = "The `close()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close)"]
    pub fn close_with_code_and_reason(
        this: &WebSocket,
        code: u16,
        reason: &str,
    ) -> Result<(), JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WebSocket" , js_name = send)]
    #[doc = "The `send()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/send)"]
    pub fn send_with_str(this: &WebSocket, data: &str) -> Result<(), JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WebSocket" , js_name = send)]
    #[doc = "The `send()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/send)"]
    pub fn send_with_blob(this: &WebSocket, data: &web_sys::Blob) -> Result<(), JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WebSocket" , js_name = send)]
    #[doc = "The `send()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/send)"]
    pub fn send_with_array_buffer(
        this: &WebSocket,
        data: &::js_sys::ArrayBuffer,
    ) -> Result<(), JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WebSocket" , js_name = send)]
    #[doc = "The `send()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/send)"]
    pub fn send_with_array_buffer_view(
        this: &WebSocket,
        data: &::js_sys::Object,
    ) -> Result<(), JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WebSocket" , js_name = send)]
    #[doc = "The `send()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/send)"]
    pub fn send_with_u8_array(this: &WebSocket, data: Uint8Array) -> Result<(), JsValue>;
}
impl WebSocket {
    #[doc = "The `WebSocket.CONNECTING` const."]
    pub const CONNECTING: u16 = 0i64 as u16;
    #[doc = "The `WebSocket.OPEN` const."]
    pub const OPEN: u16 = 1u64 as u16;
    #[doc = "The `WebSocket.CLOSING` const."]
    pub const CLOSING: u16 = 2u64 as u16;
    #[doc = "The `WebSocket.CLOSED` const."]
    pub const CLOSED: u16 = 3u64 as u16;
}
