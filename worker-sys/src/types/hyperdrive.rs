use wasm_bindgen::prelude::*;

use crate::Socket;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Hyperdrive;

    #[wasm_bindgen(method, getter, js_name=connectionString, catch)]
    pub fn connect(this: &Hyperdrive) -> Result<Socket, JsValue>;

    #[wasm_bindgen(method, getter, js_name=connectionString)]
    pub fn connection_string(this: &Hyperdrive) -> String;

    #[wasm_bindgen(method, getter, js_name=host)]
    pub fn host(this: &Hyperdrive) -> String;

    #[wasm_bindgen(method, getter, js_name=port)]
    pub fn port(this: &Hyperdrive) -> u16;

    #[wasm_bindgen(method, getter, js_name=user)]
    pub fn user(this: &Hyperdrive) -> String;

    #[wasm_bindgen(method, getter, js_name=password)]
    pub fn password(this: &Hyperdrive) -> String;

    #[wasm_bindgen(method, getter, js_name=database)]
    pub fn database(this: &Hyperdrive) -> String;
}
