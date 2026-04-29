#[allow(unused_imports)]
use js_sys::*;
#[allow(unused_imports)]
use wasm_bindgen::prelude::*;
#[allow(dead_code)]
use JsValue as T;
#[allow(dead_code)]
pub type FlagshipEvaluationContext = Object;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type FlagshipEvaluationDetails;
    #[wasm_bindgen(method, getter, js_name = "flagKey")]
    pub fn flag_key(this: &FlagshipEvaluationDetails) -> String;
    #[wasm_bindgen(method, setter, js_name = "flagKey")]
    pub fn set_flag_key(this: &FlagshipEvaluationDetails, val: &str);
    #[wasm_bindgen(method, getter)]
    pub fn value(this: &FlagshipEvaluationDetails) -> T;
    #[wasm_bindgen(method, setter)]
    pub fn set_value(this: &FlagshipEvaluationDetails, val: &T);
    #[wasm_bindgen(method, getter)]
    pub fn variant(this: &FlagshipEvaluationDetails) -> Option<String>;
    #[wasm_bindgen(method, setter)]
    pub fn set_variant(this: &FlagshipEvaluationDetails, val: &str);
    #[wasm_bindgen(method, setter, js_name = "variant")]
    pub fn set_variant_with_null(this: &FlagshipEvaluationDetails, val: &Null);
    #[wasm_bindgen(method, getter)]
    pub fn reason(this: &FlagshipEvaluationDetails) -> Option<String>;
    #[wasm_bindgen(method, setter)]
    pub fn set_reason(this: &FlagshipEvaluationDetails, val: &str);
    #[wasm_bindgen(method, setter, js_name = "reason")]
    pub fn set_reason_with_null(this: &FlagshipEvaluationDetails, val: &Null);
    #[wasm_bindgen(method, getter, js_name = "errorCode")]
    pub fn error_code(this: &FlagshipEvaluationDetails) -> Option<String>;
    #[wasm_bindgen(method, setter, js_name = "errorCode")]
    pub fn set_error_code(this: &FlagshipEvaluationDetails, val: &str);
    #[wasm_bindgen(method, setter, js_name = "errorCode")]
    pub fn set_error_code_with_null(this: &FlagshipEvaluationDetails, val: &Null);
    #[wasm_bindgen(method, getter, js_name = "errorMessage")]
    pub fn error_message(this: &FlagshipEvaluationDetails) -> Option<String>;
    #[wasm_bindgen(method, setter, js_name = "errorMessage")]
    pub fn set_error_message(this: &FlagshipEvaluationDetails, val: &str);
    #[wasm_bindgen(method, setter, js_name = "errorMessage")]
    pub fn set_error_message_with_null(this: &FlagshipEvaluationDetails, val: &Null);
}
impl FlagshipEvaluationDetails {
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `flag_key`"]
    #[doc = " * `value`"]
    pub fn new(flag_key: &str, value: &T) -> FlagshipEvaluationDetails {
        Self::builder(flag_key, value).build()
    }
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `flag_key`"]
    #[doc = " * `value`"]
    pub fn builder(flag_key: &str, value: &T) -> FlagshipEvaluationDetailsBuilder {
        let inner: Self = JsCast::unchecked_into(js_sys::Object::new());
        inner.set_flag_key(flag_key);
        inner.set_value(value);
        FlagshipEvaluationDetailsBuilder { inner }
    }
}
pub struct FlagshipEvaluationDetailsBuilder {
    inner: FlagshipEvaluationDetails,
}
impl FlagshipEvaluationDetailsBuilder {
    pub fn variant(self, val: &str) -> Self {
        self.inner.set_variant(val);
        self
    }
    pub fn variant_with_null(self, val: &Null) -> Self {
        self.inner.set_variant_with_null(val);
        self
    }
    pub fn reason(self, val: &str) -> Self {
        self.inner.set_reason(val);
        self
    }
    pub fn reason_with_null(self, val: &Null) -> Self {
        self.inner.set_reason_with_null(val);
        self
    }
    pub fn error_code(self, val: &str) -> Self {
        self.inner.set_error_code(val);
        self
    }
    pub fn error_code_with_null(self, val: &Null) -> Self {
        self.inner.set_error_code_with_null(val);
        self
    }
    pub fn error_message(self, val: &str) -> Self {
        self.inner.set_error_message(val);
        self
    }
    pub fn error_message_with_null(self, val: &Null) -> Self {
        self.inner.set_error_message_with_null(val);
        self
    }
    pub fn build(self) -> FlagshipEvaluationDetails {
        self.inner
    }
}
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Flagship;
    #[doc = " Get a flag value without type checking."]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `flagKey` - The key of the flag to evaluate."]
    #[doc = " * `defaultValue` - Optional default value returned when evaluation fails."]
    #[doc = " * `context` - Optional evaluation context for targeting rules."]
    #[wasm_bindgen(method, catch)]
    pub async fn get(this: &Flagship, flag_key: &str) -> Result<JsValue, JsValue>;
    #[doc = " Get a flag value without type checking."]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `flagKey` - The key of the flag to evaluate."]
    #[doc = " * `defaultValue` - Optional default value returned when evaluation fails."]
    #[doc = " * `context` - Optional evaluation context for targeting rules."]
    #[wasm_bindgen(method, catch, js_name = "get")]
    pub async fn get_with_default_value(
        this: &Flagship,
        flag_key: &str,
        default_value: &JsValue,
    ) -> Result<JsValue, JsValue>;
    #[doc = " Get a flag value without type checking."]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `flagKey` - The key of the flag to evaluate."]
    #[doc = " * `defaultValue` - Optional default value returned when evaluation fails."]
    #[doc = " * `context` - Optional evaluation context for targeting rules."]
    #[wasm_bindgen(method, catch, js_name = "get")]
    pub async fn get_with_default_value_and_context(
        this: &Flagship,
        flag_key: &str,
        default_value: &JsValue,
        context: &Object,
    ) -> Result<JsValue, JsValue>;
    #[doc = " Get a boolean flag value."]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `flagKey` - The key of the flag to evaluate."]
    #[doc = " * `defaultValue` - Default value returned when evaluation fails or the flag type does not match."]
    #[doc = " * `context` - Optional evaluation context for targeting rules."]
    #[wasm_bindgen(method, catch, js_name = "getBooleanValue")]
    pub async fn get_boolean_value(
        this: &Flagship,
        flag_key: &str,
        default_value: bool,
    ) -> Result<Boolean, JsValue>;
    #[doc = " Get a boolean flag value."]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `flagKey` - The key of the flag to evaluate."]
    #[doc = " * `defaultValue` - Default value returned when evaluation fails or the flag type does not match."]
    #[doc = " * `context` - Optional evaluation context for targeting rules."]
    #[wasm_bindgen(method, catch, js_name = "getBooleanValue")]
    pub async fn get_boolean_value_with_context(
        this: &Flagship,
        flag_key: &str,
        default_value: bool,
        context: &Object,
    ) -> Result<Boolean, JsValue>;
    #[doc = " Get a string flag value."]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `flagKey` - The key of the flag to evaluate."]
    #[doc = " * `defaultValue` - Default value returned when evaluation fails or the flag type does not match."]
    #[doc = " * `context` - Optional evaluation context for targeting rules."]
    #[wasm_bindgen(method, catch, js_name = "getStringValue")]
    pub async fn get_string_value(
        this: &Flagship,
        flag_key: &str,
        default_value: &str,
    ) -> Result<JsString, JsValue>;
    #[doc = " Get a string flag value."]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `flagKey` - The key of the flag to evaluate."]
    #[doc = " * `defaultValue` - Default value returned when evaluation fails or the flag type does not match."]
    #[doc = " * `context` - Optional evaluation context for targeting rules."]
    #[wasm_bindgen(method, catch, js_name = "getStringValue")]
    pub async fn get_string_value_with_context(
        this: &Flagship,
        flag_key: &str,
        default_value: &str,
        context: &Object,
    ) -> Result<JsString, JsValue>;
    #[doc = " Get a number flag value."]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `flagKey` - The key of the flag to evaluate."]
    #[doc = " * `defaultValue` - Default value returned when evaluation fails or the flag type does not match."]
    #[doc = " * `context` - Optional evaluation context for targeting rules."]
    #[wasm_bindgen(method, catch, js_name = "getNumberValue")]
    pub async fn get_number_value(
        this: &Flagship,
        flag_key: &str,
        default_value: f64,
    ) -> Result<Number, JsValue>;
    #[doc = " Get a number flag value."]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `flagKey` - The key of the flag to evaluate."]
    #[doc = " * `defaultValue` - Default value returned when evaluation fails or the flag type does not match."]
    #[doc = " * `context` - Optional evaluation context for targeting rules."]
    #[wasm_bindgen(method, catch, js_name = "getNumberValue")]
    pub async fn get_number_value_with_context(
        this: &Flagship,
        flag_key: &str,
        default_value: f64,
        context: &Object,
    ) -> Result<Number, JsValue>;
    #[doc = " Get a boolean flag value with full evaluation details."]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `flagKey` - The key of the flag to evaluate."]
    #[doc = " * `defaultValue` - Default value returned when evaluation fails or the flag type does not match."]
    #[doc = " * `context` - Optional evaluation context for targeting rules."]
    #[wasm_bindgen(method, catch, js_name = "getBooleanDetails")]
    pub async fn get_boolean_details(
        this: &Flagship,
        flag_key: &str,
        default_value: bool,
    ) -> Result<FlagshipEvaluationDetails, JsValue>;
    #[doc = " Get a boolean flag value with full evaluation details."]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `flagKey` - The key of the flag to evaluate."]
    #[doc = " * `defaultValue` - Default value returned when evaluation fails or the flag type does not match."]
    #[doc = " * `context` - Optional evaluation context for targeting rules."]
    #[wasm_bindgen(method, catch, js_name = "getBooleanDetails")]
    pub async fn get_boolean_details_with_context(
        this: &Flagship,
        flag_key: &str,
        default_value: bool,
        context: &Object,
    ) -> Result<FlagshipEvaluationDetails, JsValue>;
    #[doc = " Get a string flag value with full evaluation details."]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `flagKey` - The key of the flag to evaluate."]
    #[doc = " * `defaultValue` - Default value returned when evaluation fails or the flag type does not match."]
    #[doc = " * `context` - Optional evaluation context for targeting rules."]
    #[wasm_bindgen(method, catch, js_name = "getStringDetails")]
    pub async fn get_string_details(
        this: &Flagship,
        flag_key: &str,
        default_value: &str,
    ) -> Result<FlagshipEvaluationDetails, JsValue>;
    #[doc = " Get a string flag value with full evaluation details."]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `flagKey` - The key of the flag to evaluate."]
    #[doc = " * `defaultValue` - Default value returned when evaluation fails or the flag type does not match."]
    #[doc = " * `context` - Optional evaluation context for targeting rules."]
    #[wasm_bindgen(method, catch, js_name = "getStringDetails")]
    pub async fn get_string_details_with_context(
        this: &Flagship,
        flag_key: &str,
        default_value: &str,
        context: &Object,
    ) -> Result<FlagshipEvaluationDetails, JsValue>;
    #[doc = " Get a number flag value with full evaluation details."]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `flagKey` - The key of the flag to evaluate."]
    #[doc = " * `defaultValue` - Default value returned when evaluation fails or the flag type does not match."]
    #[doc = " * `context` - Optional evaluation context for targeting rules."]
    #[wasm_bindgen(method, catch, js_name = "getNumberDetails")]
    pub async fn get_number_details(
        this: &Flagship,
        flag_key: &str,
        default_value: f64,
    ) -> Result<FlagshipEvaluationDetails, JsValue>;
    #[doc = " Get a number flag value with full evaluation details."]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `flagKey` - The key of the flag to evaluate."]
    #[doc = " * `defaultValue` - Default value returned when evaluation fails or the flag type does not match."]
    #[doc = " * `context` - Optional evaluation context for targeting rules."]
    #[wasm_bindgen(method, catch, js_name = "getNumberDetails")]
    pub async fn get_number_details_with_context(
        this: &Flagship,
        flag_key: &str,
        default_value: f64,
        context: &Object,
    ) -> Result<FlagshipEvaluationDetails, JsValue>;
}
