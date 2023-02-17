use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object, js_name=DurableObjectId)]
    pub type DurableObjectId;

    #[wasm_bindgen(method, js_class=DurableObjectId, js_name=toString)]
    pub fn to_string(this: &DurableObjectId) -> String;
}
