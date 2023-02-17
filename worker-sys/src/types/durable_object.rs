use wasm_bindgen::prelude::*;

mod id;
mod namespace;
mod state;
mod storage;
mod transaction;

pub use id::*;
pub use namespace::*;
pub use state::*;
pub use storage::*;
pub use transaction::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object, js_name=DurableObject)]
    pub type DurableObject;

    #[wasm_bindgen(method, js_class=DurableObject, js_name=fetch)]
    pub fn fetch_with_request(this: &DurableObject, req: &web_sys::Request) -> js_sys::Promise;

    #[wasm_bindgen(method, js_class=DurableObject, js_name=fetch)]
    pub fn fetch_with_str(this: &DurableObject, url: &str) -> js_sys::Promise;
}
