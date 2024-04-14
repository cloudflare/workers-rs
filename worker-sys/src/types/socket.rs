use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "cloudflare:sockets")]
extern "C" {
    #[wasm_bindgen(catch)]
    pub fn connect(address: JsValue, options: JsValue) -> Result<Socket, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Socket;

    #[wasm_bindgen(method, catch)]
    pub fn close(this: &Socket) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn closed(this: &Socket) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn opened(this: &Socket) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name=startTls)]
    pub fn start_tls(this: &Socket) -> Result<Socket, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn readable(this: &Socket) -> Result<web_sys::ReadableStream, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn writable(this: &Socket) -> Result<web_sys::WritableStream, JsValue>;
}
