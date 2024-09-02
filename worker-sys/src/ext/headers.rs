use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = js_sys::Object)]
    pub type Headers;

    #[wasm_bindgen(method, js_name = getAll)]
    pub fn get_all(this: &Headers, name: &str) -> js_sys::Array;
}

pub trait HeadersExt {
    fn get_all(&self, name: &str) -> js_sys::Array;
}

impl HeadersExt for web_sys::Headers {
    fn get_all(&self, name: &str) -> js_sys::Array {
        self.unchecked_ref::<Headers>().get_all(name)
    }
}
