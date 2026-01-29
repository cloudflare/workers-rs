//! Cloudflare Workflows support for Rust Workers.

use std::future::Future;
use std::panic::AssertUnwindSafe;

use js_sys::{Object, Reflect};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{future_to_promise, JsFuture};
use worker_sys::types::WorkflowBinding as WorkflowBindingSys;
use worker_sys::types::WorkflowInstanceSys;
use worker_sys::types::WorkflowStep as WorkflowStepSys;

use crate::env::EnvBinding;
use crate::send::SendFuture;
use crate::Result;

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
        let promise = self.inner.get(id)?;
        let result = SendFuture::new(JsFuture::from(promise)).await?;
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
        let promise = self.inner.create(js_options)?;
        let result = SendFuture::new(JsFuture::from(promise)).await?;
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
        let promise = self.inner.create_batch(&js_array)?;
        let result = SendFuture::new(JsFuture::from(promise)).await?;
        let result_array: js_sys::Array = result.unchecked_into();

        let mut instances = Vec::with_capacity(result_array.length() as usize);
        for i in 0..result_array.length() {
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
        if constructor_name == Self::TYPE_NAME
            || constructor_name == "WorkflowImpl"
            || constructor_name == "Fetcher"
        {
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
    pub success_retention: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_retention: Option<String>,
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
    pub fn id(&self) -> Result<String> {
        get_string_property(self.inner.as_ref(), "id")
    }

    /// Pause the workflow instance.
    pub async fn pause(&self) -> Result<()> {
        let promise = self.inner.pause()?;
        SendFuture::new(JsFuture::from(promise)).await?;
        Ok(())
    }

    /// Resume a paused workflow instance.
    pub async fn resume(&self) -> Result<()> {
        let promise = self.inner.resume()?;
        SendFuture::new(JsFuture::from(promise)).await?;
        Ok(())
    }

    /// Terminate the workflow instance.
    pub async fn terminate(&self) -> Result<()> {
        let promise = self.inner.terminate()?;
        SendFuture::new(JsFuture::from(promise)).await?;
        Ok(())
    }

    /// Restart the workflow instance.
    pub async fn restart(&self) -> Result<()> {
        let promise = self.inner.restart()?;
        SendFuture::new(JsFuture::from(promise)).await?;
        Ok(())
    }

    /// Get the current status of the workflow instance.
    pub async fn status(&self) -> Result<InstanceStatus> {
        let promise = self.inner.status()?;
        let result = SendFuture::new(JsFuture::from(promise)).await?;
        Ok(serde_wasm_bindgen::from_value(result)?)
    }

    /// Send an event to the workflow instance to trigger `step.wait_for_event()` calls.
    pub async fn send_event<T: Serialize>(&self, event_type: &str, payload: T) -> Result<()> {
        let event = Object::new();
        Reflect::set(&event, &"type".into(), &event_type.into())?;
        Reflect::set(
            &event,
            &"payload".into(),
            &serde_wasm_bindgen::to_value(&payload)?,
        )?;
        let promise = self.inner.send_event(event.into())?;
        SendFuture::new(JsFuture::from(promise)).await?;
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

/// Provides methods for executing durable workflow steps.
#[derive(Debug)]
pub struct WorkflowStep(WorkflowStepSys);

// SAFETY: See Workflow for rationale - WASM is single-threaded.
unsafe impl Send for WorkflowStep {}
unsafe impl Sync for WorkflowStep {}

impl WorkflowStep {
    fn wrap_callback<T, F, Fut>(
        callback: F,
    ) -> wasm_bindgen::closure::Closure<dyn FnMut() -> js_sys::Promise>
    where
        T: Serialize + 'static,
        F: FnOnce() -> Fut + 'static,
        Fut: Future<Output = Result<T>> + 'static,
    {
        wasm_bindgen::closure::Closure::once(move || -> js_sys::Promise {
            future_to_promise(AssertUnwindSafe(async move {
                let result = callback().await.map_err(JsValue::from)?;
                serialize_as_object(&result).map_err(|e| JsValue::from_str(&e.to_string()))
            }))
        })
    }

    /// Execute a named step. The callback's return value is persisted and
    /// returned without re-executing on replay.
    pub async fn do_<T, F, Fut>(&self, name: &str, callback: F) -> Result<T>
    where
        T: Serialize + DeserializeOwned + 'static,
        F: FnOnce() -> Fut + 'static,
        Fut: Future<Output = Result<T>> + 'static,
    {
        let closure = Self::wrap_callback(callback);
        let js_fn = closure.as_ref().unchecked_ref::<js_sys::Function>();
        let promise = self.0.do_(name, js_fn)?;
        let result = SendFuture::new(JsFuture::from(promise)).await?;
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
        F: FnOnce() -> Fut + 'static,
        Fut: Future<Output = Result<T>> + 'static,
    {
        let config_js = serde_wasm_bindgen::to_value(&config)?;
        let closure = Self::wrap_callback(callback);
        let js_fn = closure.as_ref().unchecked_ref::<js_sys::Function>();
        let promise = self.0.do_with_config(name, config_js, js_fn)?;
        let result = SendFuture::new(JsFuture::from(promise)).await?;
        Ok(serde_wasm_bindgen::from_value(result)?)
    }

    /// Sleep for a specified duration (e.g., "1 minute", "5 seconds").
    pub async fn sleep(&self, name: &str, duration: impl Into<WorkflowDuration>) -> Result<()> {
        let duration_js = duration.into().to_js_value();
        let promise = self.0.sleep(name, duration_js)?;
        SendFuture::new(JsFuture::from(promise)).await?;
        Ok(())
    }

    /// Sleep until a specific timestamp.
    pub async fn sleep_until(&self, name: &str, timestamp: impl Into<crate::Date>) -> Result<()> {
        let date: crate::Date = timestamp.into();
        let ts_ms = date.as_millis() as f64;
        let promise = self.0.sleep_until(name, ts_ms.into())?;
        SendFuture::new(JsFuture::from(promise)).await?;
        Ok(())
    }

    /// Wait for an external event sent via `WorkflowInstance::send_event()`.
    pub async fn wait_for_event<T: DeserializeOwned>(
        &self,
        name: &str,
        options: WaitForEventOptions,
    ) -> Result<WorkflowStepEvent<T>> {
        let options_js = serde_wasm_bindgen::to_value(&options)?;
        let promise = self.0.wait_for_event(name, options_js)?;
        let result = SendFuture::new(JsFuture::from(promise)).await?;
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
    pub timeout: Option<String>,
}

/// Retry configuration for a workflow step.
#[derive(Debug, Clone, Serialize)]
pub struct RetryConfig {
    pub limit: u32,
    pub delay: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backoff: Option<Backoff>,
}

/// Backoff strategy for retries.
#[derive(Debug, Clone, Serialize)]
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
    pub event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
}

/// An event received from `wait_for_event`.
#[derive(Debug, Clone)]
pub struct WorkflowStepEvent<T> {
    pub payload: T,
    pub timestamp: crate::Date,
    pub event_type: String,
}

impl<T: DeserializeOwned> WorkflowStepEvent<T> {
    fn from_js(value: JsValue) -> Result<Self> {
        Ok(Self {
            payload: serde_wasm_bindgen::from_value(get_property(&value, "payload")?)?,
            timestamp: get_timestamp_property(&value, "timestamp")?,
            event_type: get_string_property(&value, "type")?,
        })
    }
}

/// The event passed to a workflow's run method.
#[derive(Debug, Clone)]
pub struct WorkflowEvent<T> {
    pub payload: T,
    pub timestamp: crate::Date,
    pub instance_id: String,
}

impl<T: DeserializeOwned> WorkflowEvent<T> {
    pub fn from_js(value: JsValue) -> Result<Self> {
        Ok(Self {
            payload: serde_wasm_bindgen::from_value(get_property(&value, "payload")?)?,
            timestamp: get_timestamp_property(&value, "timestamp")?,
            instance_id: get_string_property(&value, "instanceId")?,
        })
    }
}

/// Duration type for workflow sleep operations.
#[derive(Debug, Clone)]
pub enum WorkflowDuration {
    Milliseconds(u64),
    String(String),
}

impl WorkflowDuration {
    fn to_js_value(&self) -> JsValue {
        match self {
            Self::Milliseconds(ms) => JsValue::from_f64(*ms as f64),
            Self::String(s) => JsValue::from_str(s),
        }
    }
}

impl From<&str> for WorkflowDuration {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<String> for WorkflowDuration {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<std::time::Duration> for WorkflowDuration {
    fn from(d: std::time::Duration) -> Self {
        Self::Milliseconds(d.as_millis() as u64)
    }
}

/// Error type for non-retryable workflow errors.
#[derive(Debug)]
pub struct NonRetryableError {
    message: String,
    name: Option<String>,
}

impl NonRetryableError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            name: None,
        }
    }

    pub fn with_name(message: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            name: Some(name.into()),
        }
    }
}

impl std::fmt::Display for NonRetryableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{}: {}", name, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for NonRetryableError {}

/// Marker trait implemented by the `#[workflow]` macro.
#[doc(hidden)]
pub trait HasWorkflowAttribute {}

/// Trait for implementing a Workflow entrypoint.
#[allow(async_fn_in_trait)]
pub trait WorkflowEntrypoint: HasWorkflowAttribute {
    fn new(ctx: crate::Context, env: crate::Env) -> Self;

    async fn run(
        &self,
        event: WorkflowEvent<serde_json::Value>,
        step: WorkflowStep,
    ) -> Result<serde_json::Value>;
}
