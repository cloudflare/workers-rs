use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen (extends = web_sys::Blob , extends = ::js_sys::Object , js_name = File , typescript_type = "File")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `File` class."]
    pub type File;

    #[wasm_bindgen (structural , method , getter , js_class = "File" , js_name = name)]
    #[doc = "Getter for the `name` field of this object."]
    pub fn name(this: &File) -> String;

    #[wasm_bindgen (structural , method , getter , js_class = "File" , js_name = lastModified)]
    #[doc = "Getter for the `lastModified` field of this object."]
    pub fn last_modified(this: &File) -> f64;

    # [wasm_bindgen (method , structural , js_class = "File" , js_name = arrayBuffer)]
    #[doc = "The `arrayBuffer()` method."]
    pub fn array_buffer(this: &File) -> ::js_sys::Promise;

    #[wasm_bindgen(catch, constructor, js_class = "File")]
    #[doc = "The `new File(..)` constructor, creating a new instance of `File`."]
    pub fn new_with_u8_array_sequence(
        file_bits: &::wasm_bindgen::JsValue,
        file_name: &str,
    ) -> Result<File, JsValue>;

    #[wasm_bindgen(catch, constructor, js_class = "File")]
    #[doc = "The `new File(..)` constructor, creating a new instance of `File`."]
    pub fn new_with_u8_array_sequence_and_options(
        file_bits: &::wasm_bindgen::JsValue,
        file_name: &str,
        options: &FilePropertyBag,
    ) -> Result<File, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = ::js_sys::Object , js_name = FilePropertyBag)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `FilePropertyBag` dictionary."]
    pub type FilePropertyBag;
}
impl FilePropertyBag {
    #[doc = "Construct a new `FilePropertyBag`."]
    pub fn new() -> Self {
        ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new())
    }

    #[doc = "Change the `lastModified` field of this object."]
    pub fn last_modified(&mut self, val: f64) -> &mut Self {
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("lastModified"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }

    #[doc = "Change the `type` field of this object."]
    pub fn type_(&mut self, val: &str) -> &mut Self {
        let r = ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("type"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
}
impl Default for FilePropertyBag {
    fn default() -> Self {
        Self::new()
    }
}
