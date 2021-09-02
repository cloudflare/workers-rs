use wasm_bindgen::prelude::*;

use crate::Cf;
use crate::RequestInit;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=Request)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Request;

    #[wasm_bindgen(structural, method, getter, js_class=Request, js_name=method)]
    pub fn method(this: &Request) -> String;

    #[wasm_bindgen(structural, method, getter, js_class=Request, js_name=url)]
    pub fn url(this: &Request) -> String;

    #[wasm_bindgen(structural, method, getter, js_class=Request, js_name=headers)]
    pub fn headers(this: &Request) -> crate::headers::Headers;

    #[wasm_bindgen(structural, method, getter, js_class=Request, js_name=redirect)]
    pub fn redirect(this: &Request) -> web_sys::RequestRedirect;

    #[wasm_bindgen(structural, method, getter, js_class=Request, js_name=bodyUsed)]
    pub fn body_used(this: &Request) -> bool;

    #[wasm_bindgen(structural, method, getter, js_class=Request, js_name=body)]
    pub fn body(this: &Request) -> Option<web_sys::ReadableStream>;

    #[wasm_bindgen(catch, constructor, js_class=Request)]
    pub fn new_with_request(input: &Request) -> Result<Request, JsValue>;

    #[wasm_bindgen(catch, constructor, js_class=Request)]
    pub fn new_with_str(input: &str) -> Result<Request, JsValue>;

    #[wasm_bindgen(catch, constructor, js_class=Request)]
    pub fn new_with_request_and_init(
        input: &Request,
        init: &RequestInit,
    ) -> Result<Request, JsValue>;

    #[wasm_bindgen(catch, constructor, js_class=Request)]
    pub fn new_with_str_and_init(input: &str, init: &RequestInit) -> Result<Request, JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class=Request, js_name=clone)]
    pub fn clone(this: &Request) -> Result<Request, JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class=Request, js_name=arrayBuffer)]
    pub fn array_buffer(this: &Request) -> Result<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class=Request, js_name=formData)]
    pub fn form_data(this: &Request) -> Result<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class=Request, js_name=json)]
    pub fn json(this: &Request) -> Result<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class=Request, js_name=text)]
    pub fn text(this: &Request) -> Result<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(structural, method, getter, js_class=Request, js_name=cf)]
    pub fn cf(this: &Request) -> Cf;
}
