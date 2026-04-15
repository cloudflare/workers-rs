use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "cloudflare:email")]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type EmailMessage;

    #[wasm_bindgen(constructor, catch)]
    pub fn new(from: &str, to: &str, raw: &str) -> Result<EmailMessage, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type SendEmail;

    #[wasm_bindgen(method, catch)]
    pub fn send(this: &SendEmail, message: &EmailMessage) -> Result<js_sys::Promise, JsValue>;
}
