use wasm_bindgen::prelude::*;

mod id;
mod namespace;
mod state;
mod storage;
mod transaction;
mod sql_storage;

pub use id::*;
pub use namespace::*;
pub use state::*;
pub use storage::*;
pub use transaction::*;
pub use sql_storage::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
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
