use wasm_bindgen::prelude::*;

mod glue {
    use super::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(extends=js_sys::Object)]
        pub type Headers;

        #[wasm_bindgen(method)]
        pub fn entries(this: &Headers) -> js_sys::Iterator;
    }
}

pub trait HeadersExt {
    fn entries(&self) -> js_sys::Iterator;
}

impl HeadersExt for web_sys::Headers {
    fn entries(&self) -> js_sys::Iterator {
        self.unchecked_ref::<glue::Headers>().entries()
    }
}
