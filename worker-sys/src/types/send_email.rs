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

    // The runtime's `send` overload accepts either an `EmailMessage` instance
    // or a plain builder object, so the arg is declared as `JsValue` and the
    // caller is responsible for constructing the right shape.
    #[wasm_bindgen(method, catch)]
    pub fn send(this: &SendEmail, message: &JsValue) -> Result<js_sys::Promise, JsValue>;
}
