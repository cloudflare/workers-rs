use crate::{send::SendFuture, EnvBinding, Result};
use js_sys::{JsString, Object, Promise};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

// Hand-written companion to `flagship_gen.rs`. ts-gen erases
// `<T extends object>` to `JsValue`, so the typed `get_object_*` methods
// live here with serde conversions folded in. `EvaluationContext` and the
// `EnvBinding` impl are also here because ts-gen doesn't synthesize them.
pub use crate::flagship_gen::{Flagship, FlagshipEvaluationDetails};

impl EnvBinding for Flagship {
    const TYPE_NAME: &'static str = "Flagship";

    // Miniflare's `wrappedBindings` expose the binding as a plain `Object`,
    // so the default `constructor.name` check fails under local dev.
    fn get(val: JsValue) -> Result<Self> {
        Ok(val.unchecked_into())
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(method, js_name = "getObjectValue")]
    fn get_object_value_raw(this: &Flagship, flag_key: &str, default_value: &JsValue) -> Promise;

    #[wasm_bindgen(method, js_name = "getObjectValue")]
    fn get_object_value_with_record_raw(
        this: &Flagship,
        flag_key: &str,
        default_value: &JsValue,
        context: &Object<JsString>,
    ) -> Promise;

    #[wasm_bindgen(method, js_name = "getObjectDetails")]
    fn get_object_details_raw(this: &Flagship, flag_key: &str, default_value: &JsValue)
        -> Promise;

    #[wasm_bindgen(method, js_name = "getObjectDetails")]
    fn get_object_details_with_record_raw(
        this: &Flagship,
        flag_key: &str,
        default_value: &JsValue,
        context: &Object<JsString>,
    ) -> Promise;
}

impl Flagship {
    /// Evaluate an object-typed flag, returning the resolved value
    /// deserialized into `T`.
    pub async fn get_object_value<T: Serialize + DeserializeOwned>(
        &self,
        flag_key: &str,
        default_value: &T,
    ) -> Result<T> {
        let default = serde_wasm_bindgen::to_value(default_value)?;
        let raw = SendFuture::new(JsFuture::from(self.get_object_value_raw(flag_key, &default)))
            .await?;
        Ok(serde_wasm_bindgen::from_value(raw)?)
    }

    /// Evaluate an object-typed flag with a targeting context.
    pub async fn get_object_value_with_record<T: Serialize + DeserializeOwned>(
        &self,
        flag_key: &str,
        default_value: &T,
        context: &EvaluationContext,
    ) -> Result<T> {
        let default = serde_wasm_bindgen::to_value(default_value)?;
        let promise =
            self.get_object_value_with_record_raw(flag_key, &default, context.as_record());
        let raw = SendFuture::new(JsFuture::from(promise)).await?;
        Ok(serde_wasm_bindgen::from_value(raw)?)
    }

    /// Evaluate an object-typed flag and return the full evaluation
    /// envelope (variant, reason, error code) with `value` deserialized
    /// into `T`.
    pub async fn get_object_details<T: Serialize + DeserializeOwned>(
        &self,
        flag_key: &str,
        default_value: &T,
    ) -> Result<EvaluationDetails<T>> {
        let default = serde_wasm_bindgen::to_value(default_value)?;
        let raw = SendFuture::new(JsFuture::from(
            self.get_object_details_raw(flag_key, &default),
        ))
        .await?;
        Ok(serde_wasm_bindgen::from_value(raw)?)
    }

    /// Evaluate an object-typed flag with a targeting context, returning
    /// the full evaluation envelope.
    pub async fn get_object_details_with_record<T: Serialize + DeserializeOwned>(
        &self,
        flag_key: &str,
        default_value: &T,
        context: &EvaluationContext,
    ) -> Result<EvaluationDetails<T>> {
        let default = serde_wasm_bindgen::to_value(default_value)?;
        let promise =
            self.get_object_details_with_record_raw(flag_key, &default, context.as_record());
        let raw = SendFuture::new(JsFuture::from(promise)).await?;
        Ok(serde_wasm_bindgen::from_value(raw)?)
    }
}

/// Typed evaluation record returned by [`Flagship::get_object_details`].
/// For boolean / string / number flags, the auto-generated
/// [`FlagshipEvaluationDetails`] is used instead.
///
/// `error_code` and `error_message` are only populated when evaluation
/// fell back to `default_value`.
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

/// Evaluation attributes passed to Flagship for targeting rules. Values are
/// constrained to `string`, `number`, and `boolean` to match the JS
/// `Record<string, string | number | boolean>`.
///
/// Pass via the auto-generated `_with_record` method variants, e.g.
/// [`Flagship::get_boolean_value_with_record`], or via the hand-written
/// [`Flagship::get_object_value_with_record`].
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    inner: Object,
}

impl Default for EvaluationContext {
    fn default() -> Self {
        Self::new()
    }
}

impl EvaluationContext {
    pub fn new() -> Self {
        Self {
            inner: Object::new(),
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

    /// The `Object<JsString>` phantom is a compile-time tag for the
    /// `_with_record` extern variants — at runtime the same JS object
    /// satisfies the `Object<Number>` and `Object<Boolean>` overloads too.
    pub fn as_record(&self) -> &Object<JsString> {
        self.inner.unchecked_ref()
    }
}
