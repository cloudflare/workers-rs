use wasm_bindgen::prelude::*;

mod id;
mod namespace;
mod sql_storage;
mod state;
mod storage;
mod transaction;

pub use id::*;
pub use namespace::*;
pub use sql_storage::*;
pub use state::*;
pub use storage::*;
pub use transaction::*;

#[wasm_bindgen(module = "cloudflare:workers")]
extern "C" {
    #[wasm_bindgen(js_name = "DurableObject")]
    #[derive(Clone)]
    pub type DurableObject;

    #[wasm_bindgen(method, catch, js_name=fetch)]
    pub fn fetch_with_request(
        this: &DurableObject,
        req: &web_sys::Request,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name=fetch)]
    pub fn fetch_with_str(this: &DurableObject, url: &str) -> Result<js_sys::Promise, JsValue>;
}
