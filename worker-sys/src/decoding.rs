#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = TextDecoder , typescript_type = "TextDecoder")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `TextDecoder` class."]
    pub type TextDecoder;

    # [wasm_bindgen (structural , method , getter , js_class = "TextDecoder" , js_name = encoding)]
    #[doc = "Getter for the `encoding` field of this object."]
    pub fn encoding(this: &TextDecoder) -> String;

    # [wasm_bindgen (structural , method , getter , js_class = "TextDecoder" , js_name = fatal)]
    #[doc = "Getter for the `fatal` field of this object."]
    pub fn fatal(this: &TextDecoder) -> bool;

    #[wasm_bindgen(catch, constructor, js_class = "TextDecoder")]
    #[doc = "The `new TextDecoder(..)` constructor, creating a new instance of `TextDecoder`."]
    pub fn new() -> Result<TextDecoder, JsValue>;

    #[wasm_bindgen(catch, constructor, js_class = "TextDecoder")]
    #[doc = "The `new TextDecoder(..)` constructor, creating a new instance of `TextDecoder`."]
    pub fn new_with_label(label: &str) -> Result<TextDecoder, JsValue>;

    #[wasm_bindgen(catch, constructor, js_class = "TextDecoder")]
    #[doc = "The `new TextDecoder(..)` constructor, creating a new instance of `TextDecoder`."]
    pub fn new_with_label_and_options(
        label: &str,
        options: &TextDecoderOptions,
    ) -> Result<TextDecoder, JsValue>;

    # [wasm_bindgen (catch , method , structural , js_class = "TextDecoder" , js_name = decode)]
    #[doc = "The `decode()` method."]
    pub fn decode(this: &TextDecoder) -> Result<String, JsValue>;

    # [wasm_bindgen (catch , method , structural , js_class = "TextDecoder" , js_name = decode)]
    #[doc = "The `decode()` method."]
    pub fn decode_with_u8_array(this: &TextDecoder, input: &mut [u8]) -> Result<String, JsValue>;

    # [wasm_bindgen (catch , method , structural , js_class = "TextDecoder" , js_name = decode)]
    #[doc = "The `decode()` method."]
    pub fn decode_with_u16_array(this: &TextDecoder, input: &mut [u16]) -> Result<String, JsValue>;

    # [wasm_bindgen (catch , method , structural , js_class = "TextDecoder" , js_name = decode)]
    #[doc = "The `decode()` method."]
    pub fn decode_with_u8_array_and_options(
        this: &TextDecoder,
        input: &mut [u8],
        options: &TextDecodeOptions,
    ) -> Result<String, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = TextDecoderOptions)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `TextDecoderOptions` dictionary."]
    pub type TextDecoderOptions;
}

impl TextDecoderOptions {
    #[doc = "Construct a new `TextDecoderOptions`."]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }

    #[doc = "Change the `fatal` field of this object."]
    pub fn fatal(&mut self, val: bool) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("fatal"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
}

impl Default for TextDecoderOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = TextDecodeOptions)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `TextDecodeOptions` dictionary."]
    pub type TextDecodeOptions;
}

impl TextDecodeOptions {
    #[doc = "Construct a new `TextDecodeOptions`."]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }

    #[doc = "Change the `stream` field of this object."]
    pub fn stream(&mut self, val: bool) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r =
            ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("stream"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
}

impl Default for TextDecodeOptions {
    fn default() -> Self {
        Self::new()
    }
}
