use wasm_bindgen::prelude::*;

mod glue {
    use super::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(extends=js_sys::Object)]
        pub type Headers;

        #[wasm_bindgen(method, catch)]
        pub fn entries(this: &Headers) -> Result<js_sys::Iterator, JsValue>;

        #[wasm_bindgen(method, catch)]
        pub fn keys(this: &Headers) -> Result<js_sys::Iterator, JsValue>;

        #[wasm_bindgen(method, catch)]
        pub fn values(this: &Headers) -> Result<js_sys::Iterator, JsValue>;
    }
}

pub trait HeadersExt {
    fn entries(&self) -> Result<js_sys::Iterator, JsValue>;

    fn keys(&self) -> Result<js_sys::Iterator, JsValue>;

    fn values(&self) -> Result<js_sys::Iterator, JsValue>;
}

impl HeadersExt for web_sys::Headers {
    fn entries(&self) -> Result<js_sys::Iterator, JsValue> {
        self.unchecked_ref::<glue::Headers>().entries()
    }

    fn keys(&self) -> Result<js_sys::Iterator, JsValue> {
        self.unchecked_ref::<glue::Headers>().keys()
    }

    fn values(&self) -> Result<js_sys::Iterator, JsValue> {
        self.unchecked_ref::<glue::Headers>().values()
    }
}
