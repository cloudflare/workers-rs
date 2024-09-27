use js_sys::Promise;
use wasm_bindgen::prelude::*;
use web_sys::{Headers, ReadableStream};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Clone, PartialEq, Eq)]
    pub type EmailMessage;

    // TODO(lduarte): see if also accepting string is needed
    #[wasm_bindgen(constructor, catch)]
    pub fn new(from: &str, to: &str, raw: &str) -> Result<EmailMessage, JsValue>;

    #[wasm_bindgen(method, getter)]
    pub fn from(this: &EmailMessage) -> String;

    #[wasm_bindgen(method, getter)]
    pub fn to(this: &EmailMessage) -> String;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Clone, PartialEq, Eq)]
    pub type ForwardableEmailMessage;

    #[wasm_bindgen(method, getter)]
    pub fn from(this: &ForwardableEmailMessage) -> String;

    #[wasm_bindgen(method, getter)]
    pub fn to(this: &ForwardableEmailMessage) -> String;

    #[wasm_bindgen(method, getter)]
    pub fn raw(this: &ForwardableEmailMessage) -> ReadableStream;

    // File size will never pass over 4GB so u32 is enough
    #[wasm_bindgen(method, getter, js_name=rawSize)]
    pub fn raw_size(this: &ForwardableEmailMessage) -> u32;

    #[wasm_bindgen(method, catch, js_name=setReject)]
    pub fn set_reject(this: &ForwardableEmailMessage, reason: &str) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn forward(
        this: &ForwardableEmailMessage,
        rcpt_to: &str,
        headers: Headers,
    ) -> Result<Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn reply(this: &ForwardableEmailMessage, email: EmailMessage) -> Result<Promise, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Clone, PartialEq, Eq)]
    pub type SendEmail;

    #[wasm_bindgen(method, catch)]
    pub fn send(this: &SendEmail, email: EmailMessage) -> Result<Promise, JsValue>;

}
