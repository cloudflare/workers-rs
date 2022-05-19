#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = TextEncoder , typescript_type = "TextEncoder")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `TextEncoder` class."]
    pub type TextEncoder;

    # [wasm_bindgen (structural , method , getter , js_class = "TextEncoder" , js_name = encoding)]
    #[doc = "Getter for the `encoding` field of this object."]
    pub fn encoding(this: &TextEncoder) -> String;

    #[wasm_bindgen(catch, constructor, js_class = "TextEncoder")]
    #[doc = "The `new TextEncoder(..)` constructor, creating a new instance of `TextEncoder`."]
    pub fn new() -> Result<TextEncoder, JsValue>;

    # [wasm_bindgen (method , structural , js_class = "TextEncoder" , js_name = encode)]
    #[doc = "The `encode()` method."]
    pub fn encode_with_input(this: &TextEncoder, input: &str) -> Vec<u8>;
}
