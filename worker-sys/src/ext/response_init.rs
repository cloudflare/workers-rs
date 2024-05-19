use wasm_bindgen::prelude::*;

pub trait ResponseInitExt {
    /// Change the `webSocket` field of this object.
    fn websocket(&mut self, val: &web_sys::WebSocket) -> Result<&mut Self, JsValue>;

    /// Change the `encodeBody` field of this object.
    fn encode_body(&mut self, val: &str) -> Result<&mut Self, JsValue>;

    /// Change the `cf` field of this object.
    fn cf(&mut self, val: &JsValue) -> Result<&mut Self, JsValue>;
}

impl ResponseInitExt for web_sys::ResponseInit {
    fn websocket(&mut self, val: &web_sys::WebSocket) -> Result<&mut Self, JsValue> {
        js_sys::Reflect::set(self.as_ref(), &JsValue::from("webSocket"), val.as_ref())?;
        Ok(self)
    }

    fn encode_body(&mut self, val: &str) -> Result<&mut Self, JsValue> {
        js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("encodeBody"),
            &JsValue::from(val),
        )?;
        Ok(self)
    }

    fn cf(&mut self, val: &JsValue) -> Result<&mut Self, JsValue> {
        js_sys::Reflect::set(self.as_ref(), &JsValue::from("cf"), val)?;
        Ok(self)
    }
}
