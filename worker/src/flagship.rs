use crate::{send::SendFuture, EnvBinding, Error, Result};
use js_sys::Object;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use worker_sys::Flagship as FlagshipSys;

/// A binding to a [Cloudflare Flagship](https://developers.cloudflare.com/flagship/) feature-flag
/// store. Retrieved via [`Env::flagship`](crate::Env::flagship).
///
/// The `get_*_value` methods never surface evaluation failures as errors. If anything goes wrong
/// (missing flag, type mismatch, evaluation error) they just hand back the `default_value` you
/// passed in. Reach for the `_details` variants when you need to see `error_code`,
/// `error_message`, or `reason`.
#[derive(Debug)]
pub struct Flagship(FlagshipSys);

unsafe impl Send for Flagship {}
unsafe impl Sync for Flagship {}

impl EnvBinding for Flagship {
    const TYPE_NAME: &'static str = "Flagship";

    // Miniflare's `wrappedBindings` expose the binding as a plain `Object`, so the default
    // constructor-name check fails under local dev. Skip it; a mismatched binding will blow up
    // loudly enough on the first method call. Mirrors `AnalyticsEngineDataset::get`.
    fn get(val: JsValue) -> Result<Self> {
        let obj = Object::from(val);
        Ok(obj.unchecked_into())
    }
}

impl JsCast for Flagship {
    fn instanceof(val: &JsValue) -> bool {
        val.is_instance_of::<FlagshipSys>()
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        Self(val.into())
    }

    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}

impl AsRef<JsValue> for Flagship {
    fn as_ref(&self) -> &JsValue {
        &self.0
    }
}

impl From<Flagship> for JsValue {
    fn from(flagship: Flagship) -> Self {
        JsValue::from(flagship.0)
    }
}

impl From<FlagshipSys> for Flagship {
    fn from(inner: FlagshipSys) -> Self {
        Self(inner)
    }
}

impl Flagship {
    /// Evaluate a flag without a compile-time type constraint. The return type is inferred from
    /// the caller's turbofish or assignment.
    pub async fn get<T: DeserializeOwned>(
        &self,
        flag_key: &str,
        default_value: impl Serialize,
        context: Option<&EvaluationContext>,
    ) -> Result<T> {
        let default = serde_wasm_bindgen::to_value(&default_value)?;
        let promise = self
            .0
            .get(flag_key, default, EvaluationContext::as_js(context));
        let out = SendFuture::new(JsFuture::from(promise)).await?;
        Ok(serde_wasm_bindgen::from_value(out)?)
    }

    pub async fn get_boolean_value(
        &self,
        flag_key: &str,
        default_value: bool,
        context: Option<&EvaluationContext>,
    ) -> Result<bool> {
        let promise =
            self.0
                .get_boolean_value(flag_key, default_value, EvaluationContext::as_js(context));
        let out = SendFuture::new(JsFuture::from(promise)).await?;
        out.as_bool()
            .ok_or_else(|| Error::RustError("expected boolean from Flagship".into()))
    }

    pub async fn get_string_value(
        &self,
        flag_key: &str,
        default_value: &str,
        context: Option<&EvaluationContext>,
    ) -> Result<String> {
        let promise =
            self.0
                .get_string_value(flag_key, default_value, EvaluationContext::as_js(context));
        let out = SendFuture::new(JsFuture::from(promise)).await?;
        out.as_string()
            .ok_or_else(|| Error::RustError("expected string from Flagship".into()))
    }

    pub async fn get_number_value(
        &self,
        flag_key: &str,
        default_value: f64,
        context: Option<&EvaluationContext>,
    ) -> Result<f64> {
        let promise =
            self.0
                .get_number_value(flag_key, default_value, EvaluationContext::as_js(context));
        let out = SendFuture::new(JsFuture::from(promise)).await?;
        out.as_f64()
            .ok_or_else(|| Error::RustError("expected number from Flagship".into()))
    }

    pub async fn get_object_value<T: Serialize + DeserializeOwned>(
        &self,
        flag_key: &str,
        default_value: &T,
        context: Option<&EvaluationContext>,
    ) -> Result<T> {
        let default = serde_wasm_bindgen::to_value(default_value)?;
        let promise = self
            .0
            .get_object_value(flag_key, default, EvaluationContext::as_js(context));
        let out = SendFuture::new(JsFuture::from(promise)).await?;
        Ok(serde_wasm_bindgen::from_value(out)?)
    }

    pub async fn get_boolean_details(
        &self,
        flag_key: &str,
        default_value: bool,
        context: Option<&EvaluationContext>,
    ) -> Result<EvaluationDetails<bool>> {
        let promise =
            self.0
                .get_boolean_details(flag_key, default_value, EvaluationContext::as_js(context));
        let out = SendFuture::new(JsFuture::from(promise)).await?;
        Ok(serde_wasm_bindgen::from_value(out)?)
    }

    pub async fn get_string_details(
        &self,
        flag_key: &str,
        default_value: &str,
        context: Option<&EvaluationContext>,
    ) -> Result<EvaluationDetails<String>> {
        let promise =
            self.0
                .get_string_details(flag_key, default_value, EvaluationContext::as_js(context));
        let out = SendFuture::new(JsFuture::from(promise)).await?;
        Ok(serde_wasm_bindgen::from_value(out)?)
    }

    pub async fn get_number_details(
        &self,
        flag_key: &str,
        default_value: f64,
        context: Option<&EvaluationContext>,
    ) -> Result<EvaluationDetails<f64>> {
        let promise =
            self.0
                .get_number_details(flag_key, default_value, EvaluationContext::as_js(context));
        let out = SendFuture::new(JsFuture::from(promise)).await?;
        Ok(serde_wasm_bindgen::from_value(out)?)
    }

    pub async fn get_object_details<T: Serialize + DeserializeOwned>(
        &self,
        flag_key: &str,
        default_value: &T,
        context: Option<&EvaluationContext>,
    ) -> Result<EvaluationDetails<T>> {
        let default = serde_wasm_bindgen::to_value(default_value)?;
        let promise =
            self.0
                .get_object_details(flag_key, default, EvaluationContext::as_js(context));
        let out = SendFuture::new(JsFuture::from(promise)).await?;
        Ok(serde_wasm_bindgen::from_value(out)?)
    }
}

/// Evaluation attributes passed to Flagship for targeting rules. Values are constrained to
/// `string`, `number`, and `boolean` to match the JS `Record<string, string | number | boolean>`.
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    inner: js_sys::Object,
}

impl Default for EvaluationContext {
    fn default() -> Self {
        Self::new()
    }
}

impl EvaluationContext {
    pub fn new() -> Self {
        Self {
            inner: js_sys::Object::new(),
        }
    }

    pub fn string(self, key: &str, value: &str) -> Self {
        self.set(key, &JsValue::from_str(value));
        self
    }

    pub fn number(self, key: &str, value: f64) -> Self {
        self.set(key, &JsValue::from_f64(value));
        self
    }

    pub fn bool(self, key: &str, value: bool) -> Self {
        self.set(key, &JsValue::from_bool(value));
        self
    }

    fn set(&self, key: &str, value: &JsValue) {
        let _ = js_sys::Reflect::set(&self.inner, &JsValue::from_str(key), value);
    }

    /// Convert an optional context into the `JsValue` the JS bindings expect (`undefined` when
    /// absent).
    fn as_js(context: Option<&Self>) -> JsValue {
        context.map_or(JsValue::UNDEFINED, |c| c.inner.clone().into())
    }
}

impl AsRef<JsValue> for EvaluationContext {
    fn as_ref(&self) -> &JsValue {
        self.inner.as_ref()
    }
}

/// Full evaluation record returned by the `get_*_details` methods.
///
/// `reason` says why Flagship picked the value it did (e.g. `"TARGETING_MATCH"`, `"DEFAULT"`).
/// `error_code` and `error_message` are only set when evaluation fell back to the default;
/// `"TYPE_MISMATCH"` and `"GENERAL"` are the codes you'll see most often.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationDetails<T> {
    pub flag_key: String,
    pub value: T,
    #[serde(default)]
    pub variant: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub error_code: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
}
