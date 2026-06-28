//! Cloudflare Workflows support for Rust Workers.
//!
//! ## `Send`/`Sync` for JS-backed types
//!
//! Several types in this module wrap JS objects and have `unsafe impl Send` /
//! `unsafe impl Sync`. WASM is single-threaded and these JS objects only ever
//! live on the main thread; the impls exist solely so the types can be held
//! across `await` points in async machinery that demands `Send`. Cross-thread
//! access is impossible in the Workers runtime.

use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::rc::Rc;

use js_sys::{Object, Reflect};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::future_to_promise;
use worker_sys::types::{
    NonRetryableErrorSys, WorkflowBinding as WorkflowBindingSys, WorkflowInstanceSys,
    WorkflowStep as WorkflowStepSys,
};

use crate::env::EnvBinding;
use crate::send::SendFuture;
use crate::Result;

/// Serialize a value to a JS object, with maps serialized as plain objects
/// rather than `Map` instances.
///
/// This exists for the macro-generated entrypoint to convert
/// [`WorkflowEntrypoint::Output`] into the JS shape the Workflows runtime
/// expects. It's not part of the user-facing API.
#[doc(hidden)]
pub fn serialize_as_object<T: Serialize>(
    value: &T,
) -> std::result::Result<JsValue, serde_wasm_bindgen::Error> {
    value.serialize(&serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true))
}

fn get_property(target: &JsValue, name: &str) -> Result<JsValue> {
    Reflect::get(target, &JsValue::from_str(name))
        .map_err(|e| crate::Error::JsError(format!("failed to get property '{name}': {e:?}")))
}

fn get_string_property(target: &JsValue, name: &str) -> Result<String> {
    get_property(target, name)?
        .as_string()
        .ok_or_else(|| crate::Error::JsError(format!("{name} is not a string")))
}

fn get_timestamp_property(target: &JsValue, name: &str) -> Result<crate::Date> {
    let val = get_property(target, name)?;
    Ok(crate::Date::from(js_sys::Date::from(val)))
}

fn get_f64_property(target: &JsValue, name: &str) -> Option<f64> {
    get_property(target, name).ok().and_then(|v| v.as_f64())
}

/// A Workflow binding for creating and managing workflow instances.
#[derive(Debug, Clone)]
pub struct Workflow {
    inner: WorkflowBindingSys,
}

// SAFETY: see module-level docs.
unsafe impl Send for Workflow {}
unsafe impl Sync for Workflow {}

impl Workflow {
    /// Get a handle to an existing workflow instance by ID.
    pub async fn get(&self, id: &str) -> Result<WorkflowInstance> {
        let result = SendFuture::new(self.inner.get(id)).await?;
        Ok(WorkflowInstance::from_js(result))
    }

    /// Create a new workflow instance with the given options.
    pub async fn create<T: Serialize>(
        &self,
        options: CreateOptions<T>,
    ) -> Result<WorkflowInstance> {
        let js_options = serialize_create_options(&options)?;
        let result = SendFuture::new(self.inner.create(js_options)).await?;
        Ok(WorkflowInstance::from_js(result))
    }

    /// Create a new workflow instance with no params and a runtime-generated id.
    pub async fn create_default(&self) -> Result<WorkflowInstance> {
        self.create(CreateOptions::<()>::default()).await
    }

    /// Create a batch of workflow instances (limited to 100 at a time).
    pub async fn create_batch<T: Serialize>(
        &self,
        batch: Vec<CreateOptions<T>>,
    ) -> Result<Vec<WorkflowInstance>> {
        let js_array = batch
            .iter()
            .map(|opts| serialize_create_options(opts))
            .collect::<Result<js_sys::Array>>()?;
        let result = SendFuture::new(self.inner.create_batch(&js_array)).await?;
        let result_array: js_sys::Array = result.unchecked_into();
        Ok(result_array.iter().map(WorkflowInstance::from_js).collect())
    }
}

// Serializer used everywhere we hand a payload off to the Workflows runtime.
//
// - `serialize_missing_as_null(true)`: the runtime destructures `{ params = {} }`
//   from the create options, so a missing (or `undefined`) `params` becomes `{}`,
//   which isn't nullish and can't be deserialized as `()` for `Input = ()` workflows.
// - `serialize_maps_as_objects(true)`: deserializing a JS `Map` back into a Rust
//   struct fails (the struct deserializer reads named fields off an object;
//   Maps don't expose entries as own properties). Emitting plain objects keeps
//   the round-trip working for `serde_json::Value::Object`, `HashMap`, etc.
fn workflow_serializer() -> serde_wasm_bindgen::Serializer {
    serde_wasm_bindgen::Serializer::new()
        .serialize_missing_as_null(true)
        .serialize_maps_as_objects(true)
}

fn serialize_create_options<T: Serialize>(options: &CreateOptions<T>) -> Result<JsValue> {
    options
        .serialize(&workflow_serializer())
        .map_err(Into::into)
}

impl EnvBinding for Workflow {
    const TYPE_NAME: &'static str = "Workflow";

    fn get(val: JsValue) -> Result<Self> {
        let obj = Object::from(val);
        let constructor_name = obj.constructor().name();
        if constructor_name == Self::TYPE_NAME || constructor_name == "WorkflowImpl" {
            Ok(Self {
                inner: obj.unchecked_into(),
            })
        } else {
            Err(format!(
                "Binding cannot be cast to the type {} from {}",
                Self::TYPE_NAME,
                constructor_name
            )
            .into())
        }
    }
}

impl JsCast for Workflow {
    fn instanceof(_val: &JsValue) -> bool {
        true
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        Self {
            inner: val.unchecked_into(),
        }
    }

    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}

impl From<Workflow> for JsValue {
    fn from(workflow: Workflow) -> Self {
        workflow.inner.into()
    }
}

impl AsRef<JsValue> for Workflow {
    fn as_ref(&self) -> &JsValue {
        self.inner.as_ref()
    }
}

/// Options for creating a new workflow instance.
#[derive(Debug, Clone, Serialize)]
pub struct CreateOptions<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    // Always serialize `params` (as `null` when `None`) so the runtime sees an
    // explicit value rather than falling through to its `params = {}` default.
    pub params: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retention: Option<RetentionOptions>,
}

impl<T> Default for CreateOptions<T> {
    fn default() -> Self {
        Self {
            id: None,
            params: None,
            retention: None,
        }
    }
}

/// Retention policy for workflow instances.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_retention: Option<WorkflowSleepDuration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_retention: Option<WorkflowSleepDuration>,
}

/// A handle to a workflow instance.
#[derive(Debug, Clone)]
pub struct WorkflowInstance {
    inner: WorkflowInstanceSys,
}

// SAFETY: see module-level docs.
unsafe impl Send for WorkflowInstance {}
unsafe impl Sync for WorkflowInstance {}

impl WorkflowInstance {
    fn from_js(val: JsValue) -> Self {
        Self {
            inner: val.unchecked_into(),
        }
    }

    /// The unique ID of this workflow instance.
    pub fn id(&self) -> String {
        get_string_property(self.inner.as_ref(), "id")
            .expect("WorkflowInstance always has an id property")
    }

    /// Pause the workflow instance.
    pub async fn pause(&self) -> Result<()> {
        SendFuture::new(self.inner.pause()).await?;
        Ok(())
    }

    /// Resume a paused workflow instance.
    pub async fn resume(&self) -> Result<()> {
        SendFuture::new(self.inner.resume()).await?;
        Ok(())
    }

    /// Terminate the workflow instance.
    pub async fn terminate(&self) -> Result<()> {
        SendFuture::new(self.inner.terminate()).await?;
        Ok(())
    }

    /// Restart the workflow instance from the beginning.
    pub async fn restart(&self) -> Result<()> {
        SendFuture::new(self.inner.restart()).await?;
        Ok(())
    }

    /// Restart the workflow instance, optionally from a specific step.
    ///
    /// When [`RestartOptions::from`] is set, cached results for all steps before
    /// the named step are preserved and execution resumes from that step.
    pub async fn restart_with_options(&self, options: RestartOptions) -> Result<()> {
        let options_js = options.serialize(&workflow_serializer())?;
        SendFuture::new(self.inner.restart_with_options(options_js)).await?;
        Ok(())
    }

    /// Get the current status of the workflow instance.
    pub async fn status(&self) -> Result<InstanceStatus> {
        let result = SendFuture::new(self.inner.status()).await?;
        Ok(serde_wasm_bindgen::from_value(result)?)
    }

    /// Send an event to the workflow instance to trigger `step.wait_for_event()` calls.
    pub async fn send_event<T: Serialize>(&self, type_: &str, payload: T) -> Result<()> {
        #[derive(Serialize)]
        struct SendEventPayload<'a, P: Serialize> {
            #[serde(rename = "type")]
            type_: &'a str,
            payload: P,
        }
        let event = SendEventPayload { type_, payload }.serialize(&workflow_serializer())?;
        SendFuture::new(self.inner.send_event(event)).await?;
        Ok(())
    }
}

/// The status of a workflow instance.
#[derive(Debug, Clone, Deserialize)]
pub struct InstanceStatus {
    pub status: InstanceStatusKind,
    #[serde(default)]
    pub error: Option<InstanceError>,
    #[serde(default)]
    pub output: Option<serde_json::Value>,
}

/// The possible status values for a workflow instance.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InstanceStatusKind {
    Queued,
    Running,
    Paused,
    Errored,
    Terminated,
    Complete,
    Waiting,
    WaitingForPause,
    Unknown,
}

/// Error information for a failed workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceError {
    pub name: String,
    pub message: String,
}

/// Options for [`WorkflowInstance::restart_with_options`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct RestartOptions {
    /// Restart from a specific step. If `None`, the instance restarts from the
    /// beginning. The step must exist in the instance's execution history.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<RestartFrom>,
}

/// Identifies the step to restart a workflow instance from.
#[derive(Debug, Clone, Serialize)]
pub struct RestartFrom {
    /// The step name as defined in your workflow code.
    pub name: String,
    /// 1-indexed occurrence of this step name. Use when the same step name
    /// appears multiple times (e.g. in a loop). Defaults to `1`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    /// Step type filter. Use when different step types share the same name.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<StepType>,
}

/// The kind of a workflow step, used to disambiguate steps in [`RestartFrom`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepType {
    #[serde(rename = "do")]
    Do,
    #[serde(rename = "sleep")]
    Sleep,
    #[serde(rename = "waitForEvent")]
    WaitForEvent,
}

/// Context passed to step callbacks with information about the current step invocation.
#[derive(Debug, Clone)]
pub struct WorkflowStepContext {
    /// Identity of the step being executed.
    pub step: WorkflowStepInfo,
    /// The current retry attempt number (starts at 1).
    pub attempt: u32,
    /// The fully resolved step configuration, with runtime defaults applied.
    pub config: ResolvedStepConfig,
}

/// Identity of a step within a workflow run.
#[derive(Debug, Clone)]
pub struct WorkflowStepInfo {
    /// The step's name, as passed to `step.do()` / `step.do_with_config()`.
    pub name: String,
    /// Number of times this step has been invoked in the current run. Starts at 1.
    ///
    /// Useful for disambiguating steps inside loops.
    pub count: u32,
}

impl WorkflowStepInfo {
    fn from_js(val: &JsValue) -> Self {
        Self {
            name: get_property(val, "name")
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_default(),
            count: get_f64_property(val, "count").unwrap_or(1.0) as u32,
        }
    }
}

impl Default for WorkflowStepInfo {
    fn default() -> Self {
        Self {
            name: String::new(),
            count: 1,
        }
    }
}

impl WorkflowStepContext {
    fn from_js(ctx: &JsValue) -> Self {
        Self {
            step: get_property(ctx, "step")
                .ok()
                .map(|v| WorkflowStepInfo::from_js(&v))
                .unwrap_or_default(),
            attempt: get_f64_property(ctx, "attempt").unwrap_or(1.0) as u32,
            config: get_property(ctx, "config")
                .ok()
                .map(|v| ResolvedStepConfig::from_js(&v))
                .unwrap_or_default(),
        }
    }
}

/// Context passed to a step's rollback (compensation) handler.
///
/// `T` is the step's output type. See [`WorkflowStep::do_with_rollback`].
#[derive(Debug, Clone)]
pub struct WorkflowRollbackContext<T> {
    /// The original context of the step being rolled back.
    pub ctx: WorkflowStepContext,
    /// The error that triggered the rollback.
    pub error: StepError,
    /// The output the step produced, if it completed successfully before the
    /// downstream failure. `None` if the step itself never produced output.
    pub output: Option<T>,
}

impl<T: DeserializeOwned> WorkflowRollbackContext<T> {
    fn from_js(value: &JsValue) -> std::result::Result<Self, JsValue> {
        let ctx = WorkflowStepContext::from_js(&get_property(value, "ctx")?);
        let error = StepError::from_js(&get_property(value, "error")?);
        let output_val = get_property(value, "output")?;
        let output = if output_val.is_undefined() || output_val.is_null() {
            None
        } else {
            Some(serde_wasm_bindgen::from_value(output_val).map_err(JsValue::from)?)
        };
        Ok(Self { ctx, error, output })
    }
}

/// The error that triggered a step rollback, mirroring a JS `Error`.
#[derive(Debug, Clone)]
pub struct StepError {
    pub name: String,
    pub message: String,
}

impl StepError {
    fn from_js(value: &JsValue) -> Self {
        let read = |key: &str| {
            get_property(value, key)
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_default()
        };
        Self {
            name: read("name"),
            message: read("message"),
        }
    }
}

impl std::fmt::Display for StepError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.message)
    }
}

impl std::error::Error for StepError {}

/// A step configuration with runtime defaults applied.
#[derive(Debug, Clone)]
pub struct ResolvedStepConfig {
    pub timeout: WorkflowSleepDuration,
    pub retries: ResolvedRetryConfig,
    /// Set when the step was marked sensitive via [`StepConfig::sensitive`].
    pub sensitive: Option<StepSensitivity>,
}

impl ResolvedStepConfig {
    fn from_js(val: &JsValue) -> Self {
        let defaults = Self::default();
        let timeout = get_property(val, "timeout")
            .ok()
            .and_then(|v| WorkflowSleepDuration::from_js_value(&v))
            .unwrap_or(defaults.timeout);
        let retries = get_property(val, "retries")
            .ok()
            .map(|v| ResolvedRetryConfig::from_js(&v))
            .unwrap_or(defaults.retries);
        let sensitive = get_property(val, "sensitive")
            .ok()
            .and_then(|v| serde_wasm_bindgen::from_value(v).ok());
        Self {
            timeout,
            retries,
            sensitive,
        }
    }
}

impl Default for ResolvedStepConfig {
    fn default() -> Self {
        Self {
            timeout: WorkflowSleepDuration::new(10, WorkflowDuration::Minutes),
            retries: ResolvedRetryConfig::default(),
            sensitive: None,
        }
    }
}

/// A retry configuration with runtime defaults applied.
#[derive(Debug, Clone)]
pub struct ResolvedRetryConfig {
    pub limit: u32,
    pub delay: WorkflowSleepDuration,
    pub backoff: Backoff,
}

impl ResolvedRetryConfig {
    fn from_js(val: &JsValue) -> Self {
        let defaults = Self::default();
        let limit = get_f64_property(val, "limit")
            .map(|v| v as u32)
            .unwrap_or(defaults.limit);
        let delay = get_property(val, "delay")
            .ok()
            .and_then(|v| WorkflowSleepDuration::from_js_value(&v))
            .unwrap_or(defaults.delay);
        let backoff = get_property(val, "backoff")
            .ok()
            .and_then(|v| serde_wasm_bindgen::from_value(v).ok())
            .unwrap_or(defaults.backoff);
        Self {
            limit,
            delay,
            backoff,
        }
    }
}

impl Default for ResolvedRetryConfig {
    fn default() -> Self {
        Self {
            limit: 5,
            delay: WorkflowSleepDuration::new(1, WorkflowDuration::Seconds),
            backoff: Backoff::Exponential,
        }
    }
}

/// Provides methods for executing durable workflow steps.
#[derive(Debug)]
pub struct WorkflowStep(WorkflowStepSys);

// SAFETY: see module-level docs.
unsafe impl Send for WorkflowStep {}
unsafe impl Sync for WorkflowStep {}

impl WorkflowStep {
    fn wrap_callback<T, F, Fut>(
        callback: F,
    ) -> wasm_bindgen::closure::Closure<dyn FnMut(JsValue) -> js_sys::Promise>
    where
        T: Serialize + 'static,
        F: Fn(WorkflowStepContext) -> Fut + 'static,
        Fut: Future<Output = Result<T>> + 'static,
    {
        Self::wrap_callback_raw(move |ctx| {
            let fut = callback(ctx);
            async move {
                let result = fut.await?;
                serialize_as_object(&result).map_err(|e| crate::Error::from(e.to_string()))
            }
        })
    }

    fn wrap_callback_raw<F, Fut>(
        callback: F,
    ) -> wasm_bindgen::closure::Closure<dyn FnMut(JsValue) -> js_sys::Promise>
    where
        F: Fn(WorkflowStepContext) -> Fut + 'static,
        Fut: Future<Output = Result<JsValue>> + 'static,
    {
        let callback = Rc::new(AssertUnwindSafe(callback));
        wasm_bindgen::closure::Closure::new(move |ctx: JsValue| -> js_sys::Promise {
            let callback = callback.clone();
            let context = WorkflowStepContext::from_js(&ctx);
            future_to_promise(AssertUnwindSafe(async move {
                (callback.0)(context).await.map_err(JsValue::from)
            }))
        })
    }

    fn wrap_stream_callback<F, Fut>(
        callback: F,
    ) -> wasm_bindgen::closure::Closure<dyn FnMut(JsValue) -> js_sys::Promise>
    where
        F: Fn(WorkflowStepContext) -> Fut + 'static,
        Fut: Future<Output = Result<web_sys::ReadableStream>> + 'static,
    {
        Self::wrap_callback_raw(move |ctx| {
            let fut = callback(ctx);
            async move { fut.await.map(JsValue::from) }
        })
    }

    /// Wrap a rollback handler into a JS function and hand it to the JS GC.
    ///
    /// Unlike step callbacks (invoked synchronously during the `do_*` await),
    /// a rollback handler runs later, when a downstream step fails,
    /// after `run` has already returned. The closure must therefore outlive
    /// this call, so we leak it via `into_js_value` rather than dropping it at
    /// the end of the step.
    fn wrap_rollback_callback<T, RF, RFut>(rollback: RF) -> JsValue
    where
        T: DeserializeOwned + 'static,
        RF: Fn(WorkflowRollbackContext<T>) -> RFut + 'static,
        RFut: Future<Output = Result<()>> + 'static,
    {
        let rollback = Rc::new(AssertUnwindSafe(rollback));
        let closure = wasm_bindgen::closure::Closure::<dyn FnMut(JsValue) -> js_sys::Promise>::new(
            move |ctx: JsValue| -> js_sys::Promise {
                let rollback = rollback.clone();
                future_to_promise(AssertUnwindSafe(async move {
                    let context = WorkflowRollbackContext::<T>::from_js(&ctx)?;
                    (rollback.0)(context).await.map_err(JsValue::from)?;
                    Ok(JsValue::UNDEFINED)
                }))
            },
        );
        closure.into_js_value()
    }

    /// Execute a named step. The callback's return value is persisted and
    /// returned without re-executing on replay.
    pub async fn do_<T, F, Fut>(&self, name: &str, callback: F) -> Result<T>
    where
        T: Serialize + DeserializeOwned + 'static,
        F: Fn(WorkflowStepContext) -> Fut + 'static,
        Fut: Future<Output = Result<T>> + 'static,
    {
        let closure = Self::wrap_callback(callback);
        let js_fn = closure.as_ref().unchecked_ref::<js_sys::Function>();
        let result = SendFuture::new(self.0.do_(name, js_fn)).await?;
        Ok(serde_wasm_bindgen::from_value(result)?)
    }

    /// Execute a named step with retry and timeout configuration.
    pub async fn do_with_config<T, F, Fut>(
        &self,
        name: &str,
        config: StepConfig,
        callback: F,
    ) -> Result<T>
    where
        T: Serialize + DeserializeOwned + 'static,
        F: Fn(WorkflowStepContext) -> Fut + 'static,
        Fut: Future<Output = Result<T>> + 'static,
    {
        let config_js = serde_wasm_bindgen::to_value(&config)?;
        let closure = Self::wrap_callback(callback);
        let js_fn = closure.as_ref().unchecked_ref::<js_sys::Function>();
        let result = SendFuture::new(self.0.do_with_config(name, config_js, js_fn)).await?;
        Ok(serde_wasm_bindgen::from_value(result)?)
    }

    /// Execute a named step with a saga-style rollback (compensation) handler.
    ///
    /// If a later step in the workflow fails, the runtime invokes the
    /// `rollback` handler to undo this step's side effects. Handlers run in
    /// reverse order of step completion. The handler receives a
    /// [`WorkflowRollbackContext`] with the original step context, the error
    /// that triggered the rollback, and this step's output.
    ///
    /// Use [`Rollback::new`] for a handler with default retry/timeout, or
    /// [`Rollback::with_config`] to customize them.
    pub async fn do_with_rollback<T, F, Fut, RF, RFut>(
        &self,
        name: &str,
        callback: F,
        rollback: Rollback<RF>,
    ) -> Result<T>
    where
        T: Serialize + DeserializeOwned + 'static,
        F: Fn(WorkflowStepContext) -> Fut + 'static,
        Fut: Future<Output = Result<T>> + 'static,
        RF: Fn(WorkflowRollbackContext<T>) -> RFut + 'static,
        RFut: Future<Output = Result<()>> + 'static,
    {
        self.do_rollback_inner(name, StepConfig::default(), callback, rollback)
            .await
    }

    /// Execute a named step with retry/timeout configuration and a saga-style
    /// rollback (compensation) handler. See [`do_with_rollback`] for rollback
    /// semantics.
    ///
    /// [`do_with_rollback`]: WorkflowStep::do_with_rollback
    pub async fn do_with_config_and_rollback<T, F, Fut, RF, RFut>(
        &self,
        name: &str,
        config: StepConfig,
        callback: F,
        rollback: Rollback<RF>,
    ) -> Result<T>
    where
        T: Serialize + DeserializeOwned + 'static,
        F: Fn(WorkflowStepContext) -> Fut + 'static,
        Fut: Future<Output = Result<T>> + 'static,
        RF: Fn(WorkflowRollbackContext<T>) -> RFut + 'static,
        RFut: Future<Output = Result<()>> + 'static,
    {
        self.do_rollback_inner(name, config, callback, rollback)
            .await
    }

    async fn do_rollback_inner<T, F, Fut, RF, RFut>(
        &self,
        name: &str,
        config: StepConfig,
        callback: F,
        rollback: Rollback<RF>,
    ) -> Result<T>
    where
        T: Serialize + DeserializeOwned + 'static,
        F: Fn(WorkflowStepContext) -> Fut + 'static,
        Fut: Future<Output = Result<T>> + 'static,
        RF: Fn(WorkflowRollbackContext<T>) -> RFut + 'static,
        RFut: Future<Output = Result<()>> + 'static,
    {
        let config_js = serde_wasm_bindgen::to_value(&config)?;
        let closure = Self::wrap_callback(callback);
        let js_fn = closure.as_ref().unchecked_ref::<js_sys::Function>();

        let rollback_options = Object::new();
        let rollback_fn = Self::wrap_rollback_callback(rollback.handler);
        Reflect::set(
            &rollback_options,
            &JsValue::from_str("rollback"),
            &rollback_fn,
        )
        .map_err(|e| crate::Error::JsError(format!("failed to set rollback: {e:?}")))?;
        if let Some(rollback_config) = rollback.config {
            let cfg_js = serde_wasm_bindgen::to_value(&rollback_config)?;
            Reflect::set(
                &rollback_options,
                &JsValue::from_str("rollbackConfig"),
                &cfg_js,
            )
            .map_err(|e| crate::Error::JsError(format!("failed to set rollbackConfig: {e:?}")))?;
        }

        let result = SendFuture::new(self.0.do_with_rollback(
            name,
            config_js,
            js_fn,
            rollback_options.into(),
        ))
        .await?;
        Ok(serde_wasm_bindgen::from_value(result)?)
    }

    /// Execute a named step whose return value is a `ReadableStream`.
    ///
    /// Unlike serialized step outputs, stream return values aren't subject to
    /// the 1 MiB payload limit, so this works for passing large bodies between
    /// steps (for example, an R2 object body).
    pub async fn do_stream<F, Fut>(
        &self,
        name: &str,
        callback: F,
    ) -> Result<web_sys::ReadableStream>
    where
        F: Fn(WorkflowStepContext) -> Fut + 'static,
        Fut: Future<Output = Result<web_sys::ReadableStream>> + 'static,
    {
        let closure = Self::wrap_stream_callback(callback);
        let js_fn = closure.as_ref().unchecked_ref::<js_sys::Function>();
        let result = SendFuture::new(self.0.do_(name, js_fn)).await?;
        Ok(result.unchecked_into())
    }

    /// Execute a named step whose return value is a `ReadableStream`, with
    /// retry and timeout configuration.
    pub async fn do_stream_with_config<F, Fut>(
        &self,
        name: &str,
        config: StepConfig,
        callback: F,
    ) -> Result<web_sys::ReadableStream>
    where
        F: Fn(WorkflowStepContext) -> Fut + 'static,
        Fut: Future<Output = Result<web_sys::ReadableStream>> + 'static,
    {
        let config_js = serde_wasm_bindgen::to_value(&config)?;
        let closure = Self::wrap_stream_callback(callback);
        let js_fn = closure.as_ref().unchecked_ref::<js_sys::Function>();
        let result = SendFuture::new(self.0.do_with_config(name, config_js, js_fn)).await?;
        Ok(result.unchecked_into())
    }

    /// Sleep for a specified duration (e.g., "1 minute", "5 seconds").
    pub async fn sleep(
        &self,
        name: &str,
        duration: impl Into<WorkflowSleepDuration>,
    ) -> Result<()> {
        let duration_js = duration.into().to_js_value();
        SendFuture::new(self.0.sleep(name, duration_js)).await?;
        Ok(())
    }

    /// Sleep until a specific timestamp.
    pub async fn sleep_until(&self, name: &str, timestamp: impl Into<crate::Date>) -> Result<()> {
        let date: crate::Date = timestamp.into();
        let ts_ms = date.as_millis() as f64;
        SendFuture::new(self.0.sleep_until(name, ts_ms.into())).await?;
        Ok(())
    }

    /// Wait for an external event sent via `WorkflowInstance::send_event()`.
    pub async fn wait_for_event<T: DeserializeOwned>(
        &self,
        name: &str,
        options: WaitForEventOptions,
    ) -> Result<WorkflowStepEvent<T>> {
        let options_js = serde_wasm_bindgen::to_value(&options)?;
        let result = SendFuture::new(self.0.wait_for_event(name, options_js)).await?;
        WorkflowStepEvent::from_js(result)
    }
}

// Macro-internal: the `#[workflow]` proc-macro feeds the JS-side `WorkflowStep`
// in via this conversion. Not part of the user-facing API.
#[doc(hidden)]
impl From<WorkflowStepSys> for WorkflowStep {
    fn from(inner: WorkflowStepSys) -> Self {
        Self(inner)
    }
}

/// Configuration for a workflow step.
#[derive(Debug, Clone, Default, Serialize)]
pub struct StepConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<RetryConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<WorkflowSleepDuration>,
    /// Marks the step's output as sensitive so it is redacted from observability.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensitive: Option<StepSensitivity>,
}

/// Marks data attached to a step as sensitive so it is redacted from logs and
/// the observability UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StepSensitivity {
    /// Redact the step's output.
    Output,
}

/// Retry configuration for a workflow step.
#[derive(Debug, Clone, Serialize)]
pub struct RetryConfig {
    pub limit: u32,
    pub delay: WorkflowSleepDuration,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backoff: Option<Backoff>,
}

/// Backoff strategy for retries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Backoff {
    Constant,
    Linear,
    Exponential,
}

/// Retry and timeout configuration for a step's rollback handler.
///
/// This is the rollback counterpart of [`StepConfig`]; only `retries` and
/// `timeout` apply to rollback handlers.
#[derive(Debug, Clone, Default, Serialize)]
pub struct RollbackConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<RetryConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<WorkflowSleepDuration>,
}

/// A rollback (compensation) handler plus its optional configuration, passed to
/// [`WorkflowStep::do_with_rollback`] / [`WorkflowStep::do_with_config_and_rollback`].
///
/// `RF` is an `async` closure of the form
/// `Fn(WorkflowRollbackContext<T>) -> impl Future<Output = Result<()>>`, where
/// `T` is the step's output type.
pub struct Rollback<RF> {
    handler: RF,
    config: Option<RollbackConfig>,
}

impl<RF> std::fmt::Debug for Rollback<RF> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rollback")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl<RF> Rollback<RF> {
    /// Create a rollback handler with default retry/timeout configuration.
    pub fn new(handler: RF) -> Self {
        Self {
            handler,
            config: None,
        }
    }

    /// Create a rollback handler with custom retry/timeout configuration.
    pub fn with_config(handler: RF, config: RollbackConfig) -> Self {
        Self {
            handler,
            config: Some(config),
        }
    }
}

/// Options for waiting for an external event.
#[derive(Debug, Clone, Serialize)]
pub struct WaitForEventOptions {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<WorkflowSleepDuration>,
}

/// An event received from `wait_for_event`.
#[derive(Debug, Clone)]
pub struct WorkflowStepEvent<T> {
    pub payload: T,
    pub timestamp: crate::Date,
    pub type_: String,
    /// Set when the event payload was marked sensitive.
    pub sensitive: Option<StepSensitivity>,
}

impl<T: DeserializeOwned> WorkflowStepEvent<T> {
    fn from_js(value: JsValue) -> Result<Self> {
        Ok(Self {
            payload: serde_wasm_bindgen::from_value(get_property(&value, "payload")?)?,
            timestamp: get_timestamp_property(&value, "timestamp")?,
            type_: get_string_property(&value, "type")?,
            sensitive: get_property(&value, "sensitive")
                .ok()
                .and_then(|v| serde_wasm_bindgen::from_value(v).ok()),
        })
    }
}

/// The event passed to a workflow's run method.
///
/// `T` matches [`WorkflowEntrypoint::Input`]; the macro deserializes the JS
/// payload into `T` before calling `run`.
#[derive(Debug, Clone)]
pub struct WorkflowEvent<T> {
    pub payload: T,
    pub timestamp: crate::Date,
    pub instance_id: String,
    /// The name of the workflow this instance belongs to.
    pub workflow_name: String,
    /// Present when the instance was triggered by a cron schedule configured
    /// via the `schedules` field of the workflow binding in `wrangler.toml`.
    pub schedule: Option<WorkflowCronSchedule>,
}

impl<T: DeserializeOwned> WorkflowEvent<T> {
    #[doc(hidden)]
    pub fn from_js(value: JsValue) -> Result<Self> {
        Ok(Self {
            payload: serde_wasm_bindgen::from_value(get_property(&value, "payload")?)?,
            timestamp: get_timestamp_property(&value, "timestamp")?,
            instance_id: get_string_property(&value, "instanceId")?,
            // Older runtimes don't populate `workflowName`; default rather than fail.
            workflow_name: get_property(&value, "workflowName")
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_default(),
            schedule: WorkflowCronSchedule::from_js(&value)?,
        })
    }
}

/// Details of the cron trigger that started a scheduled workflow instance.
///
/// Populated on [`WorkflowEvent::schedule`] when the instance was started by a
/// cron schedule declared on the workflow binding (the `schedules` field in
/// `wrangler.toml`).
#[derive(Debug, Clone)]
pub struct WorkflowCronSchedule {
    /// The cron expression that triggered this instance.
    pub cron: String,
    /// The scheduled trigger time, if the runtime reported one.
    pub scheduled_time: Option<crate::Date>,
}

impl WorkflowCronSchedule {
    /// Parse the optional `schedule` property off a workflow event object.
    fn from_js(event: &JsValue) -> Result<Option<Self>> {
        let schedule = get_property(event, "schedule")?;
        if schedule.is_undefined() || schedule.is_null() {
            return Ok(None);
        }
        let scheduled_time = get_f64_property(&schedule, "scheduledTime")
            .map(|ms| crate::Date::new(crate::DateInit::Millis(ms as u64)));
        Ok(Some(Self {
            cron: get_string_property(&schedule, "cron")?,
            scheduled_time,
        }))
    }
}

/// Unit of time for workflow durations.
#[derive(Debug, Clone, Copy)]
pub enum WorkflowDuration {
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
    Months,
    Years,
}

/// A typed duration used throughout the Workflows API for sleep, timeout,
/// retry delay, and retention fields.
///
/// Corresponds to the `WorkflowSleepDuration` type in the Workers runtime,
/// which accepts either a string like `"5 seconds"` or a number of milliseconds.
#[derive(Debug, Clone)]
enum WorkflowSleepDurationInner {
    Text(String),
    Millis(f64),
}

#[derive(Debug, Clone)]
pub struct WorkflowSleepDuration(WorkflowSleepDurationInner);

impl WorkflowSleepDuration {
    /// Create a new duration with the given amount and unit.
    pub fn new(amount: u32, unit: WorkflowDuration) -> Self {
        let unit_str = match unit {
            WorkflowDuration::Seconds => "seconds",
            WorkflowDuration::Minutes => "minutes",
            WorkflowDuration::Hours => "hours",
            WorkflowDuration::Days => "days",
            WorkflowDuration::Weeks => "weeks",
            WorkflowDuration::Months => "months",
            WorkflowDuration::Years => "years",
        };
        Self(WorkflowSleepDurationInner::Text(format!(
            "{amount} {unit_str}"
        )))
    }

    fn to_js_value(&self) -> JsValue {
        match &self.0 {
            WorkflowSleepDurationInner::Text(s) => JsValue::from_str(s),
            WorkflowSleepDurationInner::Millis(ms) => JsValue::from_f64(*ms),
        }
    }

    fn from_js_value(val: &JsValue) -> Option<Self> {
        if let Some(s) = val.as_string() {
            Some(Self(WorkflowSleepDurationInner::Text(s)))
        } else {
            val.as_f64()
                .map(|n| Self(WorkflowSleepDurationInner::Millis(n)))
        }
    }
}

impl Serialize for WorkflowSleepDuration {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        match &self.0 {
            WorkflowSleepDurationInner::Text(s) => serializer.serialize_str(s),
            WorkflowSleepDurationInner::Millis(ms) => serializer.serialize_f64(*ms),
        }
    }
}

impl From<&str> for WorkflowSleepDuration {
    fn from(s: &str) -> Self {
        Self(WorkflowSleepDurationInner::Text(s.to_string()))
    }
}

impl From<String> for WorkflowSleepDuration {
    fn from(s: String) -> Self {
        Self(WorkflowSleepDurationInner::Text(s))
    }
}

impl From<std::time::Duration> for WorkflowSleepDuration {
    fn from(d: std::time::Duration) -> Self {
        Self(WorkflowSleepDurationInner::Millis(d.as_millis() as f64))
    }
}

/// Error type for non-retryable workflow errors.
///
/// This wraps the JavaScript `NonRetryableError` from `cloudflare:workflows`,
/// which the Workflows runtime uses to identify errors that should not be retried.
#[derive(Debug)]
pub struct NonRetryableError {
    inner: NonRetryableErrorSys,
}

impl NonRetryableError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            inner: NonRetryableErrorSys::new(&message.into()),
        }
    }
}

impl std::fmt::Display for NonRetryableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.message())
    }
}

impl std::error::Error for NonRetryableError {}

impl From<NonRetryableError> for JsValue {
    fn from(e: NonRetryableError) -> Self {
        e.inner.into()
    }
}

impl From<NonRetryableError> for crate::Error {
    fn from(e: NonRetryableError) -> Self {
        crate::Error::Internal(e.inner.into())
    }
}

/// Marker trait implemented by the `#[workflow]` macro.
#[doc(hidden)]
pub trait HasWorkflowAttribute {}

/// Trait for implementing a Workflow entrypoint.
///
/// Pair this with `#[workflow]` on the struct definition. The macro generates
/// the wasm-bindgen glue and handles deserializing the incoming payload into
/// `Input` and serializing the returned `Output` back to JS.
///
/// Use `Input = ()` for workflows with no params, and `Output = ()` if you
/// don't need to return anything meaningful.
#[allow(async_fn_in_trait)]
pub trait WorkflowEntrypoint: HasWorkflowAttribute {
    /// Type of the `params` payload supplied to [`Workflow::create`].
    type Input: DeserializeOwned;

    /// Type returned to the runtime as the workflow's output.
    type Output: Serialize;

    fn new(ctx: crate::Context, env: crate::Env) -> Self;

    async fn run(
        &self,
        event: WorkflowEvent<Self::Input>,
        step: WorkflowStep,
    ) -> Result<Self::Output>;
}
