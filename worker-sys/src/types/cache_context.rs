use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type CacheContext;

    #[wasm_bindgen(method, catch)]
    pub fn purge(this: &CacheContext, options: JsValue) -> Result<js_sys::Promise, JsValue>;
}

#[wasm_bindgen(module = "cloudflare:workers")]
extern "C" {
    #[wasm_bindgen(thread_local_v2, js_name = cache)]
    pub static CACHE: CacheContext;
}
