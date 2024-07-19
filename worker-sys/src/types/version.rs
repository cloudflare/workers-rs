use wasm_bindgen::prelude::*;

/// This type was created to support https://developers.cloudflare.com/workers/runtime-apis/bindings/version-metadata/
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Clone, PartialEq, Eq)]
    pub type CfVersionMetadata;

    #[wasm_bindgen(method, getter, js_name=id)]
    pub fn id(this: &CfVersionMetadata) -> String;

    #[wasm_bindgen(method, getter, js_name=tag)]
    pub fn tag(this: &CfVersionMetadata) -> String;
}
