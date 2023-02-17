use wasm_bindgen::prelude::*;

mod glue {
    use super::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(extends=js_sys::Object, js_name=Headers)]
        pub type HeadersExt;

        #[wasm_bindgen(catch, method, structural, js_class=Headers, js_name=entries)]
        pub fn entries(this: &HeadersExt) -> Result<js_sys::Iterator, JsValue>;

        #[wasm_bindgen(catch, method, structural, js_class=Headers, js_name=keys)]
        pub fn keys(this: &HeadersExt) -> Result<js_sys::Iterator, JsValue>;

        #[wasm_bindgen(catch, method, structural, js_class=Headers, js_name=values)]
        pub fn values(this: &HeadersExt) -> Result<js_sys::Iterator, JsValue>;
    }
}

pub trait HeadersExt {
    fn entries(&self) -> Result<js_sys::Iterator, JsValue>;

    fn keys(&self) -> Result<js_sys::Iterator, JsValue>;

    fn values(&self) -> Result<js_sys::Iterator, JsValue>;
}

impl HeadersExt for web_sys::Headers {
    fn entries(&self) -> Result<js_sys::Iterator, JsValue> {
        self.unchecked_ref::<glue::HeadersExt>().entries()
    }

    fn keys(&self) -> Result<js_sys::Iterator, JsValue> {
        self.unchecked_ref::<glue::HeadersExt>().keys()
    }

    fn values(&self) -> Result<js_sys::Iterator, JsValue> {
        self.unchecked_ref::<glue::HeadersExt>().values()
    }
}
