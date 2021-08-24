use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = FormData , typescript_type = "FormData")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `FormData` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/FormData)"]
    pub type FormData;
    #[wasm_bindgen(catch, constructor, js_class = "FormData")]
    #[doc = "The `new FormData(..)` constructor, creating a new instance of `FormData`."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/FormData/FormData)"]
    pub fn new() -> Result<FormData, JsValue>;

    # [wasm_bindgen (catch , method , structural , js_class = "FormData" , js_name = append)]
    #[doc = "The `append()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/FormData/append)"]
    pub fn append_with_str(this: &FormData, name: &str, value: &str) -> Result<(), JsValue>;
    # [wasm_bindgen (method , structural , js_class = "FormData" , js_name = delete)]
    #[doc = "The `delete()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/FormData/delete)"]
    pub fn delete(this: &FormData, name: &str);
    # [wasm_bindgen (method , structural , js_class = "FormData" , js_name = get)]
    #[doc = "The `get()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/FormData/get)"]
    pub fn get(this: &FormData, name: &str) -> ::wasm_bindgen::JsValue;
    # [wasm_bindgen (method , structural , js_class = "FormData" , js_name = getAll)]
    #[doc = "The `getAll()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/FormData/getAll)"]
    pub fn get_all(this: &FormData, name: &str) -> ::js_sys::Array;
    # [wasm_bindgen (method , structural , js_class = "FormData" , js_name = has)]
    #[doc = "The `has()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/FormData/has)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `FormData`*"]
    pub fn has(this: &FormData, name: &str) -> bool;

    # [wasm_bindgen (catch , method , structural , js_class = "FormData" , js_name = set)]
    #[doc = "The `set()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/FormData/set)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `FormData`*"]
    pub fn set_with_str(this: &FormData, name: &str, value: &str) -> Result<(), JsValue>;
}
