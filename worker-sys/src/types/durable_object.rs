use wasm_bindgen::prelude::*;

mod container;
mod id;
mod namespace;
mod sql_storage;
mod state;
mod storage;
mod transaction;

pub use container::*;
pub use id::*;
pub use namespace::*;
pub use sql_storage::*;
pub use state::*;
pub use storage::*;
pub use transaction::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Clone, Debug)]
    pub type DurableObject;

    #[wasm_bindgen(method, catch, js_name=fetch)]
    pub fn fetch_with_request(
        this: &DurableObject,
        req: &web_sys::Request,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name=fetch)]
    pub fn fetch_with_str(this: &DurableObject, url: &str) -> Result<js_sys::Promise, JsValue>;

    /// Opens a TCP connection served by the object's `connect` handler.
    #[wasm_bindgen(method, catch)]
    pub fn connect(
        this: &DurableObject,
        address: &str,
        options: JsValue,
    ) -> Result<crate::types::Socket, JsValue>;
}
