use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = js_sys::Object)]
    pub type Headers;

    #[wasm_bindgen(method, catch)]
    pub fn get_all(this: &Headers, name: &str) -> Result<js_sys::Array, JsValue>;
}

pub trait HeadersExt {
    fn get_all(&self, name: &str) -> Result<js_sys::Array, JsValue>;
}

impl HeadersExt for web_sys::Headers {
    fn get_all(&self, name: &str) -> Result<js_sys::Array, JsValue> {
        self.unchecked_ref::<Headers>().get_all(name)
    }
}