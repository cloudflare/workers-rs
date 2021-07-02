use wasm_bindgen::prelude::*;

use crate::Cf;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=Request)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Request;

    #[wasm_bindgen(structural, method, getter, js_class=Request, js_name=method)]
    pub fn method(this: &Request) -> String;

    #[wasm_bindgen(structural, method, getter, js_class=Request, js_name=url)]
    pub fn url(this: &Request) -> String;

    #[cfg(feature = "Headers")]
    #[wasm_bindgen(structural, method, getter, js_class=Request, js_name=headers)]
    // #[doc = "*This API requires the following crate features to be activated: `Headers`, `Request`*"]
    pub fn headers(this: &Request) -> crate::headers::Headers;

    #[cfg(feature = "RequestRedirect")]
    #[wasm_bindgen(structural, method, getter, js_class=Request, js_name=redirect)]
    // #[doc = "*This API requires the following crate features to be activated: `Request`, `RequestRedirect`*"]
    pub fn redirect(this: &Request) -> web_sys::RequestRedirect;

    #[wasm_bindgen(structural, method, getter, js_class=Request, js_name=bodyUsed)]
    // #[doc = "*This API requires the following crate features to be activated: `Request`*"]
    pub fn body_used(this: &Request) -> bool;

    #[cfg(feature = "ReadableStream")]
    #[wasm_bindgen(structural, method, getter, js_class=Request, js_name=body)]
    // #[doc = "*This API requires the following crate features to be activated: `ReadableStream`, `Request`*"]
    pub fn body(this: &Request) -> Option<web_sys::ReadableStream>;

    #[cfg(feature = "Request")]
    #[wasm_bindgen(catch, constructor, js_class=Request)]
    // #[doc = "*This API requires the following crate features to be activated: `Request`*"]
    pub fn new_with_request(input: &Request) -> Result<Request, JsValue>;

    #[cfg(feature = "Request")]
    #[wasm_bindgen(catch, constructor, js_class=Request)]
    // #[doc = "*This API requires the following crate features to be activated: `Request`*"]
    pub fn new_with_str(input: &str) -> Result<Request, JsValue>;

    #[cfg(feature = "RequestInit")]
    #[wasm_bindgen(catch, constructor, js_class=Request)]
    // #[doc = "*This API requires the following crate features to be activated: `Request`, `RequestInit`*"]
    pub fn new_with_request_and_init(
        input: &Request,
        init: &web_sys::RequestInit,
    ) -> Result<Request, JsValue>;

    #[cfg(feature = "RequestInit")]
    #[wasm_bindgen(catch, constructor, js_class=Request)]
    // #[doc = "*This API requires the following crate features to be activated: `Request`, `RequestInit`*"]
    pub fn new_with_str_and_init(input: &str, init: &web_sys::RequestInit) -> Result<Request, JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class=Request, js_name=clone)]
    // #[doc = "*This API requires the following crate features to be activated: `Request`*"]
    pub fn clone(this: &Request) -> Result<Request, JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class=Request, js_name=arrayBuffer)]
    // #[doc = "*This API requires the following crate features to be activated: `Request`*"]
    pub fn array_buffer(this: &Request) -> Result<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class=Request, js_name=formData)]
    // #[doc = "*This API requires the following crate features to be activated: `Request`*"]
    pub fn form_data(this: &Request) -> Result<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class=Request, js_name=json)]
    // #[doc = "*This API requires the following crate features to be activated: `Request`*"]
    pub fn json(this: &Request) -> Result<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class=Request, js_name=text)]
    // #[doc = "*This API requires the following crate features to be activated: `Request`*"]
    pub fn text(this: &Request) -> Result<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(structural, method, getter, js_class=Request, js_name=cf)]
    // #[doc = "*This API requires the following crate features to be activated: `Request`, `RequestCache`*"]
    pub fn cf(this: &Request) -> Cf;
}