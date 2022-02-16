use crate::response_init::ResponseInit;
use crate::FormData;

use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=Response)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Response;

    #[wasm_bindgen(structural, method, getter, js_class=Response, js_name=url)]
    #[doc = "Getter for the `url` field of this object."]
    pub fn url(this: &Response) -> String;

    #[wasm_bindgen(structural, method, getter, js_class=Response, js_name=redirected)]
    #[doc = "Getter for the `redirected` field of this object."]
    pub fn redirected(this: &Response) -> bool;

    #[wasm_bindgen(structural, method, getter, js_class=Response, js_name=status)]
    #[doc = "Getter for the `status` field of this object."]
    pub fn status(this: &Response) -> u16;

    #[wasm_bindgen(structural, method, getter, js_class=Response, js_name=ok)]
    #[doc = "Getter for the `ok` field of this object."]
    pub fn ok(this: &Response) -> bool;

    #[wasm_bindgen(structural, method, getter, js_class=Response, js_name=statusText)]
    #[doc = "Getter for the `statusText` field of this object."]
    pub fn status_text(this: &Response) -> String;

    #[wasm_bindgen(structural, method, getter, js_class=Response, js_name=headers)]
    #[doc = "Getter for the `headers` field of this object."]
    pub fn headers(this: &Response) -> crate::headers::Headers;

    #[wasm_bindgen(structural, method, getter, js_class=Response, js_name=webSocket)]
    #[doc = "Getter for the `webSocket` field of this object."]
    pub fn websocket(this: &Response) -> Option<crate::websocket::WebSocket>;

    #[wasm_bindgen(structural, method, getter, js_class=Response, js_name=bodyUsed)]
    #[doc = "Getter for the `bodyUsed` field of this object."]
    pub fn body_used(this: &Response) -> bool;

    #[wasm_bindgen(structural, method, getter, js_class=Response, js_name=body)]
    #[doc = "Getter for the `body` field of this object."]
    pub fn body(this: &Response) -> Option<web_sys::ReadableStream>;

    #[wasm_bindgen(catch, constructor, js_class=Response)]
    #[doc = "The `new Response(..)` constructor, creating a new instance of `Response`."]
    pub fn new() -> Result<Response, JsValue>;

    #[wasm_bindgen(catch, constructor, js_class=Response)]
    #[doc = "The `new Response(..)` constructor, creating a new instance of `Response`."]
    pub fn new_with_opt_u8_array(body: Option<Uint8Array>) -> Result<Response, JsValue>;

    #[wasm_bindgen(catch, constructor, js_class=Response)]
    #[doc = "The `new Response(..)` constructor, creating a new instance of `Response`."]
    pub fn new_with_opt_form_data(body: Option<&FormData>) -> Result<Response, JsValue>;

    #[wasm_bindgen(catch, constructor, js_class=Response)]
    #[doc = "The `new Response(..)` constructor, creating a new instance of `Response`."]
    pub fn new_with_opt_str(body: Option<&str>) -> Result<Response, JsValue>;

    #[wasm_bindgen(catch, constructor, js_class=Response)]
    #[doc = "The `new Response(..)` constructor, creating a new instance of `Response`."]
    pub fn new_with_opt_stream(body: Option<&web_sys::ReadableStream>)
        -> Result<Response, JsValue>;

    #[wasm_bindgen(catch, constructor, js_class=Response)]
    #[doc = "The `new Response(..)` constructor, creating a new instance of `Response`."]
    pub fn new_with_opt_u8_array_and_init(
        body: Option<Uint8Array>,
        init: &ResponseInit,
    ) -> Result<Response, JsValue>;

    #[wasm_bindgen(catch, constructor, js_class=Response)]
    #[doc = "The `new Response(..)` constructor, creating a new instance of `Response`."]
    pub fn new_with_opt_form_data_and_init(
        body: Option<&FormData>,
        init: &ResponseInit,
    ) -> Result<Response, JsValue>;

    #[wasm_bindgen(catch, constructor, js_class=Response)]
    #[doc = "The `new Response(..)` constructor, creating a new instance of `Response`."]
    pub fn new_with_opt_str_and_init(
        body: Option<&str>,
        init: &ResponseInit,
    ) -> Result<Response, JsValue>;

    #[wasm_bindgen(catch, constructor, js_class=Response)]
    #[doc = "The `new Response(..)` constructor, creating a new instance of `Response`."]
    pub fn new_with_opt_stream_and_init(
        body: Option<web_sys::ReadableStream>,
        init: &ResponseInit,
    ) -> Result<Response, JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class=Response, js_name=clone)]
    #[doc = "The `clone()` method."]
    pub fn clone(this: &Response) -> Result<Response, JsValue>;

    #[wasm_bindgen(static_method_of=Response, js_class=Response, js_name=error)]
    #[doc = "The `error()` method."]
    pub fn error() -> Response;

    #[wasm_bindgen(catch, static_method_of=Response, js_class=Response, js_name=redirect)]
    #[doc = "The `redirect()` method."]
    pub fn redirect(url: &str) -> Result<Response, JsValue>;

    #[wasm_bindgen(catch, static_method_of=Response, js_class=Response, js_name=redirect)]
    #[doc = "The `redirect()` method."]
    pub fn redirect_with_status(url: &str, status: u16) -> Result<Response, JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class=Response, js_name=arrayBuffer)]
    #[doc = "The `arrayBuffer()` method."]
    pub fn array_buffer(this: &Response) -> Result<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class=Response, js_name=blob)]
    #[doc = "The `blob()` method."]
    pub fn blob(this: &Response) -> Result<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class=Response, js_name=formData)]
    #[doc = "The `formData()` method."]
    pub fn form_data(this: &Response) -> Result<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class=Response, js_name=json)]
    #[doc = "The `json()` method."]
    pub fn json(this: &Response) -> Result<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, structural, js_class=Response, js_name=text)]
    #[doc = "The `text()` method."]
    pub fn text(this: &Response) -> Result<::js_sys::Promise, JsValue>;
}
