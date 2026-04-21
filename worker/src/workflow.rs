//! Cloudflare Workflows support for Rust Workers.

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

/// Serialize a value to a JS object, ensuring maps are serialized as plain objects.
///
/// This is useful when returning values from [`WorkflowEntrypoint::run`] that
/// need to be plain JS objects rather than `Map` instances.
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

// SAFETY: WASM is single-threaded. These types wrap JS objects that are only
// accessed from the main thread. Send/Sync are implemented to satisfy Rust's
// async machinery (e.g., holding references across await points), but actual
// cross-thread access is impossible in the Workers runtime.
unsafe impl Send for Workflow {}
unsafe impl Sync for Workflow {}

impl Workflow {
    /// Get a handle to an existing workflow instance by ID.
    pub async fn get(&self, id: &str) -> Result<WorkflowInstance> {
        let result = SendFuture::new(self.inner.get(id)).await?;
        Ok(WorkflowInstance::from_js(result))
    }

    /// Create a new workflow instance.
    pub async fn create<T: Serialize>(
        &self,
        options: Option<CreateOptions<T>>,
    ) -> Result<WorkflowInstance> {
        let js_options = match options {
            Some(opts) => serde_wasm_bindgen::to_value(&opts)?,
            None => JsValue::UNDEFINED,
        };
        let result = SendFuture::new(self.inner.create(js_options)).await?;
        Ok(WorkflowInstance::from_js(result))
    }

    /// Create a batch of workflow instances (limited to 100 at a time).
    pub async fn create_batch<T: Serialize>(
        &self,
        batch: Vec<CreateOptions<T>>,
    ) -> Result<Vec<WorkflowInstance>> {
        let js_array = js_sys::Array::new();
        for opts in batch {
            js_array.push(&serde_wasm_bindgen::to_value(&opts)?);
        }
        let result = SendFuture::new(self.inner.create_batch(&js_array)).await?;
        let result_array: js_sys::Array = result.unchecked_into();

        let len = result_array.length();
        let mut instances = Vec::with_capacity(len as usize);
        for i in 0..len {
            instances.push(WorkflowInstance::from_js(result_array.get(i)));
        }
        Ok(instances)
    }
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
    #[serde(skip_serializing_if = "Option::is_none")]
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

// SAFETY: See Workflow for rationale - WASM is single-threaded.
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

    /// Restart the workflow instance.
    pub async fn restart(&self) -> Result<()> {
        SendFuture::new(self.inner.restart()).await?;
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
        let event = serde_wasm_bindgen::to_value(&SendEventPayload { type_, payload })?;
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
    /// Number of times this step has been invoked in the current run, starting at 1.
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

/// A step configuration with runtime defaults applied.
#[derive(Debug, Clone)]
pub struct ResolvedStepConfig {
    pub timeout: WorkflowSleepDuration,
    pub retries: ResolvedRetryConfig,
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
        Self { timeout, retries }
    }
}

impl Default for ResolvedStepConfig {
    fn default() -> Self {
        Self {
            timeout: WorkflowSleepDuration::text("10 minutes"),
            retries: ResolvedRetryConfig::default(),
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
            delay: WorkflowSleepDuration::text("1 second"),
            backoff: Backoff::Exponential,
        }
    }
}

/// Provides methods for executing durable workflow steps.
#[derive(Debug)]
pub struct WorkflowStep(WorkflowStepSys);

// SAFETY: See Workflow for rationale - WASM is single-threaded.
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

    /// Execute a named step whose return value is a `ReadableStream`.
    ///
    /// Stream return values are not subject to the 1 MiB payload limit that
    /// applies to serialized step outputs, making this suitable for passing
    /// large bodies (for example, an R2 object body) between steps.
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
}

impl<T: DeserializeOwned> WorkflowStepEvent<T> {
    fn from_js(value: JsValue) -> Result<Self> {
        Ok(Self {
            payload: serde_wasm_bindgen::from_value(get_property(&value, "payload")?)?,
            timestamp: get_timestamp_property(&value, "timestamp")?,
            type_: get_string_property(&value, "type")?,
        })
    }
}

/// The event passed to a workflow's run method.
#[derive(Debug, Clone)]
pub struct WorkflowEvent {
    pub payload: JsValue,
    pub timestamp: crate::Date,
    pub instance_id: String,
}

impl WorkflowEvent {
    pub fn from_js(value: JsValue) -> Result<Self> {
        Ok(Self {
            payload: get_property(&value, "payload")?,
            timestamp: get_timestamp_property(&value, "timestamp")?,
            instance_id: get_string_property(&value, "instanceId")?,
        })
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

    fn text(s: &str) -> Self {
        Self(WorkflowSleepDurationInner::Text(s.to_string()))
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
#[allow(async_fn_in_trait)]
pub trait WorkflowEntrypoint: HasWorkflowAttribute {
    fn new(ctx: crate::Context, env: crate::Env) -> Self;

    async fn run(&self, event: WorkflowEvent, step: WorkflowStep) -> Result<JsValue>;
}
