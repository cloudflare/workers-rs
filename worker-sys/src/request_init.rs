use wasm_bindgen::prelude::*;

use crate::abort::AbortSignal;

#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = RequestInit)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `RequestInit` dictionary."]
    pub type RequestInit;
}
impl RequestInit {
    #[doc = "Construct a new `RequestInit`."]
    pub fn new() -> Self {
        ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new())
    }
    #[doc = "Change the `body` field of this object."]
    pub fn body(&mut self, val: Option<&::wasm_bindgen::JsValue>) -> &mut Self {
        let r = ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("body"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }

    #[doc = "Change the `headers` field of this object."]
    pub fn headers(&mut self, val: &::wasm_bindgen::JsValue) -> &mut Self {
        let r = ::js_sys::Reflect::set(
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

    #[doc = "Change the `method` field of this object."]
    pub fn method(&mut self, val: &str) -> &mut Self {
        let r =
            ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("method"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }

    #[doc = "Change the `redirect` field of this object."]
    pub fn redirect(&mut self, val: RequestRedirect) -> &mut Self {
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("redirect"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }

    #[doc = "Change the `signal` field of this object."]
    pub fn signal(&mut self, val: Option<&AbortSignal>) -> &mut Self {
        let r =
            ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("signal"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
}

impl Default for RequestInit {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::manual_non_exhaustive)]
#[wasm_bindgen]
#[doc = "The `RequestRedirect` enum."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestRedirect {
    Follow = "follow",
    Error = "error",
    Manual = "manual",
}
