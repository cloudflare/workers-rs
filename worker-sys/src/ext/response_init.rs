use wasm_bindgen::prelude::*;

pub trait ResponseInitExt {
    /// Change the `webSocket` field of this object.
    fn websocket(&mut self, val: &web_sys::WebSocket) -> &mut Self;
}

impl ResponseInitExt for web_sys::ResponseInit {
    fn websocket(&mut self, val: &web_sys::WebSocket) -> &mut Self {
        let r = js_sys::Reflect::set(self.as_ref(), &JsValue::from("webSocket"), val.as_ref());
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
}
