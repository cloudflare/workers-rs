use crate::websocket::WebSocket;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = js_sys::Object , js_name = ResponseInit)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `ResponseInit` dictionary."]
    pub type ResponseInit;
}
impl ResponseInit {
    #[doc = "Construct a new `ResponseInit`."]
    pub fn new() -> Self {
        ::wasm_bindgen::JsCast::unchecked_into(js_sys::Object::new())
    }

    #[doc = "Change the `headers` field of this object."]
    pub fn headers(&mut self, val: &::wasm_bindgen::JsValue) -> &mut Self {
        let r = js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("headers"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }

    #[doc = "Change the `status` field of this object."]
    pub fn status(&mut self, val: u16) -> &mut Self {
        let r = js_sys::Reflect::set(self.as_ref(), &JsValue::from("status"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }

    #[doc = "Change the `statusText` field of this object."]
    pub fn status_text(&mut self, val: &str) -> &mut Self {
        let r = js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("statusText"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }

    #[doc = "Change the `webSocket` field of this object."]
    pub fn websocket(&mut self, val: &WebSocket) -> &mut Self {
        let r = js_sys::Reflect::set(self.as_ref(), &JsValue::from("webSocket"), val.as_ref());
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
}
impl Default for ResponseInit {
    fn default() -> Self {
        Self::new()
    }
}
