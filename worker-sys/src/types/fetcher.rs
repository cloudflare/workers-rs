use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Fetcher;

    #[wasm_bindgen(method, catch)]
    pub fn fetch(this: &Fetcher, input: &web_sys::Request) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name=fetch)]
    pub fn fetch_with_str(this: &Fetcher, input: &str) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name=fetch)]
    pub fn fetch_with_init(
        this: &Fetcher,
        input: &web_sys::Request,
        init: &web_sys::RequestInit,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name=fetch)]
    pub fn fetch_with_str_and_init(
        this: &Fetcher,
        input: &str,
        init: &web_sys::RequestInit,
    ) -> Result<js_sys::Promise, JsValue>;
}

unsafe impl Send for Fetcher {}
unsafe impl Sync for Fetcher {}
