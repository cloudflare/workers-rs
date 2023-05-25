use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "cloudflare:sockets")]
extern "C" {
    #[wasm_bindgen]
    pub fn connect(address: JsValue, options: JsValue) -> Socket;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Socket;

    #[wasm_bindgen(method)]
    pub fn close(this: &Socket) -> js_sys::Promise;

    #[wasm_bindgen(method)]
    pub fn closed(this: &Socket) -> js_sys::Promise;

    #[wasm_bindgen(method, js_name=startTls)]
    pub fn start_tls(this: &Socket) -> Socket;

    #[wasm_bindgen(method, getter = readable)]
    pub fn readable(this: &Socket) -> web_sys::ReadableStream;

    #[wasm_bindgen(method, getter = writable)]
    pub fn writable(this: &Socket) -> web_sys::WritableStream;
}
