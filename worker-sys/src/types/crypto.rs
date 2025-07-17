use wasm_bindgen::prelude::*;
use web_sys::WritableStream;

#[wasm_bindgen]
extern "C" {
    /// Bindings for the non-standard [crypto.DigestStream](https://developers.cloudflare.com/workers/runtime-apis/web-crypto/#constructors) API
    #[wasm_bindgen(extends = WritableStream)]
    #[derive(Debug)]
    pub type DigestStream;

    #[wasm_bindgen(constructor, js_namespace = crypto)]
    pub fn new(algorithm: &str) -> DigestStream;

    #[wasm_bindgen(method, getter)]
    pub fn digest(this: &DigestStream) -> js_sys::Promise;
}
