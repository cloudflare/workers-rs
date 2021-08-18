use shared::{CfProperties, RequestRedirect};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = RequestInit)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `RequestInit` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RequestInit`*"]
    pub type RequestInit;
}
impl RequestInit {
    #[doc = "Construct a new `RequestInit`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RequestInit`*"]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    #[doc = "Change the `body` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RequestInit`*"]
    pub fn body(&mut self, val: Option<&::wasm_bindgen::JsValue>) -> &mut Self {
        #[allow(unused_unsafe)]
        let r = unsafe {
            ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("body"), &JsValue::from(val))
        };
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }

    #[doc = "Change the `headers` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RequestInit`*"]
    pub fn headers(&mut self, val: &::wasm_bindgen::JsValue) -> &mut Self {
        #[allow(unused_unsafe)]
        let r = unsafe {
            ::js_sys::Reflect::set(
                self.as_ref(),
                &JsValue::from("headers"),
                &JsValue::from(val),
            )
        };
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }

    #[doc = "Change the `method` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RequestInit`*"]
    pub fn method(&mut self, val: &str) -> &mut Self {
        #[allow(unused_unsafe)]
        let r = unsafe {
            ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("method"), &JsValue::from(val))
        };
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }

    #[cfg(feature = "RequestRedirect")]
    #[doc = "Change the `redirect` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RequestInit`, `RequestRedirect`*"]
    pub fn redirect(&mut self, val: RequestRedirect) -> &mut Self {
        #[allow(unused_unsafe)]
        let r = unsafe {
            ::js_sys::Reflect::set(
                self.as_ref(),
                &JsValue::from("redirect"),
                &JsValue::from_str(val.into()),
            )
        };
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }

    #[doc = "Change the `cf` field of this object."]
    pub fn cf_properties(&mut self, props: &CfProperties) -> &mut Self {
        #[allow(unused_unsafe)]
        self
    }
}
impl Default for RequestInit {
    fn default() -> Self {
        Self::new()
    }
}
