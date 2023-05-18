use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "__STATIC_CONTENT_MANIFEST")]
extern "C" {
    #[wasm_bindgen(js_name = "default")]
    pub static STATIC_CONTENT_MANIFEST_STR: String;
}
