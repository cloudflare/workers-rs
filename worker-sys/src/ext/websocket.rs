use wasm_bindgen::prelude::*;

mod glue {
    use super::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen]
        pub type WebSocket;

        #[wasm_bindgen(method, catch)]
        pub fn accept(this: &WebSocket) -> Result<(), JsValue>;

        #[wasm_bindgen(method, catch, js_name = "serializeAttachment")]
        pub fn serialize_attachment(this: &WebSocket, value: JsValue) -> Result<(), JsValue>;

        #[wasm_bindgen(method, catch, js_name = "deserializeAttachment")]
        pub fn deserialize_attachment(this: &WebSocket) -> Result<JsValue, JsValue>;
    }
}

pub trait WebSocketExt {
    /// Accepts the server side of the WebSocket.
    ///
    /// [CF Documentation](https://developers.cloudflare.com/workers/runtime-apis/websockets#accept)
    fn accept(&self) -> Result<(), JsValue>;

    fn serialize_attachment(&self, value: JsValue) -> Result<(), JsValue>;

    fn deserialize_attachment(&self) -> Result<JsValue, JsValue>;
}

impl WebSocketExt for web_sys::WebSocket {
    fn accept(&self) -> Result<(), JsValue> {
        self.unchecked_ref::<glue::WebSocket>().accept()
    }

    fn serialize_attachment(&self, value: JsValue) -> Result<(), JsValue> {
        self.unchecked_ref::<glue::WebSocket>()
            .serialize_attachment(value)
    }

    fn deserialize_attachment(&self) -> Result<JsValue, JsValue> {
        self.unchecked_ref::<glue::WebSocket>()
            .deserialize_attachment()
    }
}
