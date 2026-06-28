use crate::{send::SendFuture, EnvBinding, Result};
use js_sys::{Object, Promise};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

// Hand-written companion to the ts-gen output in `bindings/flagship.rs`.
//
// ts-gen types `getStringValue` / `getNumberValue` (and their `*Details`
// variants) as `JsString` / `Number`. Returning plain `String` / `f64` needs
// ABI-level generics that no released wasm-bindgen has yet
// (see https://github.com/wasm-bindgen/wasm-bindgen/pull/5180), so those
// methods are hand-rolled here until it lands. The `getObject*` methods are
// here for the same reason: ts-gen erases their `<T>` to `JsValue`. They fold
// serde conversions in for typed `T`. `EvaluationContext` and the `EnvBinding`
// impl are here too, because ts-gen doesn't synthesize them.
//
// Boolean flags are fully auto-generated: ts-gen maps them onto the js_sys
// `Boolean` / `FlagshipEvaluationDetails<Boolean>` types, which work as-is.
pub use crate::bindings::flagship::{Flagship, FlagshipEvaluationDetails};

impl EnvBinding for Flagship {
    const TYPE_NAME: &'static str = "Flagship";

    // Miniflare's `wrappedBindings` expose the binding as a plain `Object`,
    // so the default `constructor.name` check fails under local dev.
    fn get(val: JsValue) -> Result<Self> {
        Ok(val.unchecked_into())
    }
}

// String / number / object methods hand-rolled off raw `Promise` externs.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(method, js_name = "getStringValue")]
    fn get_string_value_raw(this: &Flagship, flag_key: &str, default_value: &str) -> Promise;

    #[wasm_bindgen(method, js_name = "getStringValue")]
    fn get_string_value_with_context_raw(
        this: &Flagship,
        flag_key: &str,
        default_value: &str,
        context: &Object,
    ) -> Promise;

    #[wasm_bindgen(method, js_name = "getNumberValue")]
    fn get_number_value_raw(this: &Flagship, flag_key: &str, default_value: f64) -> Promise;

    #[wasm_bindgen(method, js_name = "getNumberValue")]
    fn get_number_value_with_context_raw(
        this: &Flagship,
        flag_key: &str,
        default_value: f64,
        context: &Object,
    ) -> Promise;

    #[wasm_bindgen(method, js_name = "getStringDetails")]
    fn get_string_details_raw(this: &Flagship, flag_key: &str, default_value: &str) -> Promise;

    #[wasm_bindgen(method, js_name = "getStringDetails")]
    fn get_string_details_with_context_raw(
        this: &Flagship,
        flag_key: &str,
        default_value: &str,
        context: &Object,
    ) -> Promise;

    #[wasm_bindgen(method, js_name = "getNumberDetails")]
    fn get_number_details_raw(this: &Flagship, flag_key: &str, default_value: f64) -> Promise;

    #[wasm_bindgen(method, js_name = "getNumberDetails")]
    fn get_number_details_with_context_raw(
        this: &Flagship,
        flag_key: &str,
        default_value: f64,
        context: &Object,
    ) -> Promise;

    #[wasm_bindgen(method, js_name = "getObjectValue")]
    fn get_object_value_raw(this: &Flagship, flag_key: &str, default_value: &JsValue) -> Promise;

    #[wasm_bindgen(method, js_name = "getObjectValue")]
    fn get_object_value_with_context_raw(
        this: &Flagship,
        flag_key: &str,
        default_value: &JsValue,
        context: &Object,
    ) -> Promise;

    #[wasm_bindgen(method, js_name = "getObjectDetails")]
    fn get_object_details_raw(this: &Flagship, flag_key: &str, default_value: &JsValue) -> Promise;

    #[wasm_bindgen(method, js_name = "getObjectDetails")]
    fn get_object_details_with_context_raw(
        this: &Flagship,
        flag_key: &str,
        default_value: &JsValue,
        context: &Object,
    ) -> Promise;
}

impl Flagship {
    /// Evaluate a string-typed flag.
    pub async fn get_string_value(&self, flag_key: &str, default_value: &str) -> Result<String> {
        from_js(self.get_string_value_raw(flag_key, default_value)).await
    }

    /// Evaluate a string-typed flag with a targeting context.
    pub async fn get_string_value_with_context(
        &self,
        flag_key: &str,
        default_value: &str,
        context: &Object,
    ) -> Result<String> {
        from_js(self.get_string_value_with_context_raw(flag_key, default_value, context)).await
    }

    /// Evaluate a number-typed flag.
    pub async fn get_number_value(&self, flag_key: &str, default_value: f64) -> Result<f64> {
        from_js(self.get_number_value_raw(flag_key, default_value)).await
    }

    /// Evaluate a number-typed flag with a targeting context.
    pub async fn get_number_value_with_context(
        &self,
        flag_key: &str,
        default_value: f64,
        context: &Object,
    ) -> Result<f64> {
        from_js(self.get_number_value_with_context_raw(flag_key, default_value, context)).await
    }

    /// Evaluate a string-typed flag and return the full evaluation envelope.
    pub async fn get_string_details(
        &self,
        flag_key: &str,
        default_value: &str,
    ) -> Result<EvaluationDetails<String>> {
        from_js(self.get_string_details_raw(flag_key, default_value)).await
    }

    /// Evaluate a string-typed flag with a targeting context, returning the
    /// full evaluation envelope.
    pub async fn get_string_details_with_context(
        &self,
        flag_key: &str,
        default_value: &str,
        context: &Object,
    ) -> Result<EvaluationDetails<String>> {
        from_js(self.get_string_details_with_context_raw(flag_key, default_value, context)).await
    }

    /// Evaluate a number-typed flag and return the full evaluation envelope.
    pub async fn get_number_details(
        &self,
        flag_key: &str,
        default_value: f64,
    ) -> Result<EvaluationDetails<f64>> {
        from_js(self.get_number_details_raw(flag_key, default_value)).await
    }

    /// Evaluate a number-typed flag with a targeting context, returning the
    /// full evaluation envelope.
    pub async fn get_number_details_with_context(
        &self,
        flag_key: &str,
        default_value: f64,
        context: &Object,
    ) -> Result<EvaluationDetails<f64>> {
        from_js(self.get_number_details_with_context_raw(flag_key, default_value, context)).await
    }

    /// Evaluate an object-typed flag, returning the resolved value
    /// deserialized into `T`.
    pub async fn get_object_value<T: Serialize + DeserializeOwned>(
        &self,
        flag_key: &str,
        default_value: &T,
    ) -> Result<T> {
        call_object(default_value, |default| {
            self.get_object_value_raw(flag_key, default)
        })
        .await
    }

    /// Evaluate an object-typed flag with a targeting context.
    pub async fn get_object_value_with_context<T: Serialize + DeserializeOwned>(
        &self,
        flag_key: &str,
        default_value: &T,
        context: &Object,
    ) -> Result<T> {
        call_object(default_value, |default| {
            self.get_object_value_with_context_raw(flag_key, default, context)
        })
        .await
    }

    /// Evaluate an object-typed flag and return the full evaluation
    /// envelope (variant, reason, error code) with `value` deserialized
    /// into `T`.
    pub async fn get_object_details<T: Serialize + DeserializeOwned>(
        &self,
        flag_key: &str,
        default_value: &T,
    ) -> Result<EvaluationDetails<T>> {
        call_object(default_value, |default| {
            self.get_object_details_raw(flag_key, default)
        })
        .await
    }

    /// Evaluate an object-typed flag with a targeting context, returning
    /// the full evaluation envelope.
    pub async fn get_object_details_with_context<T: Serialize + DeserializeOwned>(
        &self,
        flag_key: &str,
        default_value: &T,
        context: &Object,
    ) -> Result<EvaluationDetails<T>> {
        call_object(default_value, |default| {
            self.get_object_details_with_context_raw(flag_key, default, context)
        })
        .await
    }
}

async fn from_js<O: DeserializeOwned>(promise: Promise) -> Result<O> {
    let raw = SendFuture::new(JsFuture::from(promise)).await?;
    Ok(serde_wasm_bindgen::from_value(raw)?)
}

async fn call_object<I: Serialize, O: DeserializeOwned>(
    default_value: &I,
    promise_fn: impl FnOnce(&JsValue) -> Promise,
) -> Result<O> {
    let default = serde_wasm_bindgen::to_value(default_value)?;
    from_js(promise_fn(&default)).await
}

/// Typed evaluation record returned by the `get_*_details` methods for string,
/// number, and object flags. Boolean flags use the auto-generated
/// [`FlagshipEvaluationDetails`] instead.
///
/// `error_code` and `error_message` are only populated when evaluation
/// fell back to `default_value`.
#[derive(Debug, Clone, Deserialize, Serialize)]
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
/// Pass via `.as_ref()` to any `_with_context` method, e.g.
/// [`Flagship::get_boolean_value_with_context`] or
/// [`Flagship::get_object_value_with_context`].
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
}

impl AsRef<Object> for EvaluationContext {
    fn as_ref(&self) -> &Object {
        &self.inner
    }
}
