use wasm_bindgen::prelude::*;
use web_sys::ReadableStream;

#[wasm_bindgen(module = "cloudflare:email")]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type EmailMessage;

    #[wasm_bindgen(constructor, catch)]
    pub fn new(from: &str, to: &str, raw: &str) -> Result<EmailMessage, JsValue>;

    #[wasm_bindgen(constructor, catch)]
    pub fn new_from_stream(
        from: &str,
        to: &str,
        raw: &ReadableStream,
    ) -> Result<EmailMessage, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn from(this: &EmailMessage) -> Result<js_sys::JsString, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn to(this: &EmailMessage) -> Result<js_sys::JsString, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn headers(this: &EmailMessage) -> Result<web_sys::Headers, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn raw(this: &EmailMessage) -> Result<web_sys::ReadableStream, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=rawSize)]
    pub fn raw_size(this: &EmailMessage) -> Result<js_sys::Number, JsValue>;

    #[wasm_bindgen(method, catch, js_name=setReject)]
    pub fn set_reject(this: &EmailMessage, reason: js_sys::JsString) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn forward(
        this: &EmailMessage,
        recipient: js_sys::JsString,
        headers: Option<web_sys::Headers>,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn reply(this: &EmailMessage, message: EmailMessage) -> Result<js_sys::Promise, JsValue>;

}
