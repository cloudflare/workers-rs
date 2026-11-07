//! Cloudflare Workflows support for Rust Workers.

use std::panic::AssertUnwindSafe;
use std::rc::Rc;

use js_sys::Object;
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
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

/// Serialize a value for the Workflows runtime, JSON-compatibly like other
/// bindings' payloads:
///
/// - Maps become plain objects. A JS `Map` can't be deserialized back into a
///   struct, so `serde_json::Value::Object` or `HashMap` payloads and step
///   outputs wouldn't round-trip.
/// - `None` and `()` become `null`. The runtime destructures `{ params = {} }`
///   from the create options, so a missing `params` would arrive as `{}`,
///   which `Input = ()` workflows can't deserialize.
fn to_js<T: Serialize + ?Sized>(value: &T) -> Result<JsValue, serde_wasm_bindgen::Error> {
    value.serialize(&serde_wasm_bindgen::Serializer::json_compatible())
}

/// Deserialize a JS `Date` by passing it through as-is.
fn deserialize_date<'de, D: Deserializer<'de>>(de: D) -> Result<crate::Date, D::Error> {
    serde_wasm_bindgen::preserve::deserialize::<D, js_sys::Date>(de).map(crate::Date::from)
}

/// Deserialize an optional timestamp given in milliseconds since the Unix epoch.
fn deserialize_millis<'de, D: Deserializer<'de>>(de: D) -> Result<Option<crate::Date>, D::Error> {
    let millis = Option::<f64>::deserialize(de)?;
    Ok(millis.map(|ms| crate::Date::new(crate::DateInit::Millis(ms as u64))))
}

/// A Workflow binding for creating and managing workflow instances.
#[derive(Debug, Clone)]
pub struct Workflow {
    inner: WorkflowBindingSys,
}

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
        let result = SendFuture::new(self.inner.create(to_js(&options)?)).await?;
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
            .map(to_js)
            .collect::<Result<js_sys::Array, _>>()?;
        let result = SendFuture::new(self.inner.create_batch(&js_array)).await?;
        let result_array: js_sys::Array = result.unchecked_into();
        Ok(result_array.iter().map(WorkflowInstance::from_js).collect())
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

impl WorkflowInstance {
    fn from_js(val: JsValue) -> Self {
        Self {
            inner: val.unchecked_into(),
        }
    }

    /// The unique ID of this workflow instance.
    pub fn id(&self) -> String {
        self.inner.id()
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
        SendFuture::new(self.inner.restart_with_options(to_js(&options)?)).await?;
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
        let event = to_js(&SendEventPayload { type_, payload })?;
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
#[serde(rename_all = "camelCase")]
pub enum StepType {
    Do,
    Sleep,
    WaitForEvent,
}

/// Context passed to step callbacks with information about the current step invocation.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct WorkflowStepContext {
    /// Identity of the step being executed.
    pub step: WorkflowStepInfo,
    /// The current retry attempt number (starts at 1).
    pub attempt: u32,
    /// The fully resolved step configuration, with runtime defaults applied.
    pub config: ResolvedStepConfig,
}

impl Default for WorkflowStepContext {
    fn default() -> Self {
        Self {
            step: WorkflowStepInfo::default(),
            attempt: 1,
            config: ResolvedStepConfig::default(),
        }
    }
}

/// Identity of a step within a workflow run.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct WorkflowStepInfo {
    /// The step's name, as passed to `step.do()` / `step.do_with_config()`.
    pub name: String,
    /// Number of times this step has been invoked in the current run. Starts at 1.
    ///
    /// Useful for disambiguating steps inside loops.
    pub count: u32,
}

impl Default for WorkflowStepInfo {
    fn default() -> Self {
        Self {
            name: String::new(),
            count: 1,
        }
    }
}

/// Context passed to a step's rollback (compensation) handler.
///
/// `T` is the step's output type. See [`WorkflowStep::do_with_rollback`].
#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowRollbackContext<T> {
    /// The original context of the step being rolled back.
    #[serde(default)]
    pub ctx: WorkflowStepContext,
    /// The error that triggered the rollback.
    #[serde(default)]
    pub error: StepError,
    /// The output the step produced, if it completed successfully before the
    /// downstream failure. `None` if the step itself never produced output.
    pub output: Option<T>,
}

/// The error that triggered a step rollback, mirroring a JS `Error`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct StepError {
    pub name: String,
    pub message: String,
}

impl std::fmt::Display for StepError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.message)
    }
}

impl std::error::Error for StepError {}

/// A step configuration with runtime defaults applied.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ResolvedStepConfig {
    pub timeout: WorkflowSleepDuration,
    pub retries: ResolvedRetryConfig,
    /// Set when the step was marked sensitive via [`StepConfig::sensitive`].
    pub sensitive: Option<StepSensitivity>,
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
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ResolvedRetryConfig {
    pub limit: u32,
    pub delay: WorkflowSleepDuration,
    pub backoff: Backoff,
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

/// Wrap a step callback as the JS function `step.do` invokes on each attempt,
/// converting the callback's output to JS with `into_js`.
fn step_callback<R, F>(
    callback: F,
    into_js: fn(R) -> Result<JsValue>,
) -> Closure<dyn FnMut(JsValue) -> js_sys::Promise>
where
    R: 'static,
    F: AsyncFn(WorkflowStepContext) -> Result<R> + 'static,
{
    let callback = Rc::new(AssertUnwindSafe(callback));
    Closure::new(move |ctx: JsValue| {
        let callback = callback.clone();
        future_to_promise(AssertUnwindSafe(async move {
            // Runtimes that predate step contexts invoke the callback without one.
            let ctx = serde_wasm_bindgen::from_value::<Option<WorkflowStepContext>>(ctx)?
                .unwrap_or_default();
            Ok(into_js((callback.0)(ctx).await?)?)
        }))
    })
}

/// Wrap a rollback handler into a JS function and hand it to the JS GC.
///
/// Unlike step callbacks (invoked during the `do_*` await), a rollback handler
/// runs later, when a downstream step fails, after `run` has already returned.
/// The closure must therefore outlive the step, so it is leaked via
/// `into_js_value` rather than dropped at the end of the step.
fn rollback_callback<T, F>(rollback: F) -> JsValue
where
    T: DeserializeOwned + 'static,
    F: AsyncFn(WorkflowRollbackContext<T>) -> Result<()> + 'static,
{
    let rollback = Rc::new(AssertUnwindSafe(rollback));
    Closure::<dyn FnMut(JsValue) -> js_sys::Promise>::new(move |ctx: JsValue| {
        let rollback = rollback.clone();
        future_to_promise(AssertUnwindSafe(async move {
            (rollback.0)(serde_wasm_bindgen::from_value(ctx)?).await?;
            Ok(JsValue::UNDEFINED)
        }))
    })
    .into_js_value()
}

/// Provides methods for executing durable workflow steps.
#[derive(Debug)]
pub struct WorkflowStep(WorkflowStepSys);

impl WorkflowStep {
    /// Execute a named step. The callback's return value is persisted and
    /// returned without re-executing on replay.
    pub async fn do_<T, F>(&self, name: &str, callback: F) -> Result<T>
    where
        T: Serialize + DeserializeOwned + 'static,
        F: AsyncFn(WorkflowStepContext) -> Result<T> + 'static,
    {
        self.do_with_config(name, StepConfig::default(), callback)
            .await
    }

    /// Execute a named step with retry and timeout configuration.
    pub async fn do_with_config<T, F>(
        &self,
        name: &str,
        config: StepConfig,
        callback: F,
    ) -> Result<T>
    where
        T: Serialize + DeserializeOwned + 'static,
        F: AsyncFn(WorkflowStepContext) -> Result<T> + 'static,
    {
        let callback = step_callback(callback, |output: T| Ok(to_js(&output)?));
        let result = SendFuture::new(self.0.do_with_config(
            name,
            to_js(&config)?,
            callback.as_ref().unchecked_ref(),
        ))
        .await?;
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
    pub async fn do_with_rollback<T, F, RF>(
        &self,
        name: &str,
        callback: F,
        rollback: Rollback<RF>,
    ) -> Result<T>
    where
        T: Serialize + DeserializeOwned + 'static,
        F: AsyncFn(WorkflowStepContext) -> Result<T> + 'static,
        RF: AsyncFn(WorkflowRollbackContext<T>) -> Result<()> + 'static,
    {
        self.do_with_config_and_rollback(name, StepConfig::default(), callback, rollback)
            .await
    }

    /// Execute a named step with retry/timeout configuration and a saga-style
    /// rollback (compensation) handler. See [`do_with_rollback`] for rollback
    /// semantics.
    ///
    /// [`do_with_rollback`]: WorkflowStep::do_with_rollback
    pub async fn do_with_config_and_rollback<T, F, RF>(
        &self,
        name: &str,
        config: StepConfig,
        callback: F,
        rollback: Rollback<RF>,
    ) -> Result<T>
    where
        T: Serialize + DeserializeOwned + 'static,
        F: AsyncFn(WorkflowStepContext) -> Result<T> + 'static,
        RF: AsyncFn(WorkflowRollbackContext<T>) -> Result<()> + 'static,
    {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct RollbackOptions {
            #[serde(with = "serde_wasm_bindgen::preserve")]
            rollback: JsValue,
            #[serde(skip_serializing_if = "Option::is_none")]
            rollback_config: Option<RollbackConfig>,
        }

        let callback = step_callback(callback, |output: T| Ok(to_js(&output)?));
        let options = RollbackOptions {
            rollback: rollback_callback(rollback.handler),
            rollback_config: rollback.config,
        };
        let result = SendFuture::new(self.0.do_with_rollback(
            name,
            to_js(&config)?,
            callback.as_ref().unchecked_ref(),
            to_js(&options)?,
        ))
        .await?;
        Ok(serde_wasm_bindgen::from_value(result)?)
    }

    /// Execute a named step whose return value is a `ReadableStream`.
    ///
    /// Unlike serialized step outputs, stream return values aren't subject to
    /// the 1 MiB payload limit, so this works for passing large bodies between
    /// steps (for example, an R2 object body).
    pub async fn do_stream<F>(&self, name: &str, callback: F) -> Result<web_sys::ReadableStream>
    where
        F: AsyncFn(WorkflowStepContext) -> Result<web_sys::ReadableStream> + 'static,
    {
        self.do_stream_with_config(name, StepConfig::default(), callback)
            .await
    }

    /// Execute a named step whose return value is a `ReadableStream`, with
    /// retry and timeout configuration.
    pub async fn do_stream_with_config<F>(
        &self,
        name: &str,
        config: StepConfig,
        callback: F,
    ) -> Result<web_sys::ReadableStream>
    where
        F: AsyncFn(WorkflowStepContext) -> Result<web_sys::ReadableStream> + 'static,
    {
        let callback = step_callback(
            callback,
            |stream: web_sys::ReadableStream| Ok(stream.into()),
        );
        let result = SendFuture::new(self.0.do_with_config(
            name,
            to_js(&config)?,
            callback.as_ref().unchecked_ref(),
        ))
        .await?;
        Ok(result.unchecked_into())
    }

    /// Sleep for a specified duration (e.g., "1 minute", "5 seconds").
    pub async fn sleep(
        &self,
        name: &str,
        duration: impl Into<WorkflowSleepDuration>,
    ) -> Result<()> {
        let duration: WorkflowSleepDuration = duration.into();
        SendFuture::new(self.0.sleep(name, to_js(&duration)?)).await?;
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
        let result = SendFuture::new(self.0.wait_for_event(name, to_js(&options)?)).await?;
        Ok(serde_wasm_bindgen::from_value(result)?)
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
/// `RF` is an async closure taking a [`WorkflowRollbackContext<T>`] and
/// returning `Result<()>`, where `T` is the step's output type.
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
#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowStepEvent<T> {
    pub payload: T,
    #[serde(deserialize_with = "deserialize_date")]
    pub timestamp: crate::Date,
    #[serde(rename = "type")]
    pub type_: String,
    /// Set when the event payload was marked sensitive.
    pub sensitive: Option<StepSensitivity>,
}

/// The event passed to a workflow's run method.
///
/// `T` matches [`WorkflowEntrypoint::Input`]; the macro deserializes the JS
/// payload into `T` before calling `run`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowEvent<T> {
    pub payload: T,
    #[serde(deserialize_with = "deserialize_date")]
    pub timestamp: crate::Date,
    pub instance_id: String,
    /// The name of the workflow this instance belongs to.
    // Older runtimes don't populate `workflowName`; default rather than fail.
    #[serde(default)]
    pub workflow_name: String,
    /// Present when the instance was triggered by a cron schedule configured
    /// via the `schedules` field of the workflow binding in `wrangler.toml`.
    pub schedule: Option<WorkflowCronSchedule>,
}

/// Details of the cron trigger that started a scheduled workflow instance.
///
/// Populated on [`WorkflowEvent::schedule`] when the instance was started by a
/// cron schedule declared on the workflow binding (the `schedules` field in
/// `wrangler.toml`).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowCronSchedule {
    /// The cron expression that triggered this instance.
    pub cron: String,
    /// The scheduled trigger time, if the runtime reported one.
    #[serde(default, deserialize_with = "deserialize_millis")]
    pub scheduled_time: Option<crate::Date>,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorkflowSleepDuration(WorkflowSleepDurationInner);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum WorkflowSleepDurationInner {
    Text(String),
    Millis(f64),
}

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
        // Keeps the original JS error, which is what reaches the runtime.
        js_sys::Error::from(e.inner).into()
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

/// The `run` export generated by `#[workflow]`: deserializes the JS event for
/// [`WorkflowEntrypoint::run`] and serializes its output for the runtime.
#[doc(hidden)]
pub async fn run_workflow<W: WorkflowEntrypoint>(
    workflow: &W,
    event: JsValue,
    step: WorkflowStepSys,
) -> Result<JsValue, JsValue> {
    let event = serde_wasm_bindgen::from_value(event)?;
    let output = workflow.run(event, WorkflowStep(step)).await?;
    Ok(to_js(&output)?)
}

#[cfg(test)]
mod send_check {
    // The JS-backed handles are `Send + Sync` automatically: wasm-bindgen makes
    // `JsValue` `Send + Sync` and every extern type carries that through. Axum
    // handlers rely on it; this compile-time check guards against a regression.
    use super::{Workflow, WorkflowInstance, WorkflowStep};
    fn _assert_send_sync<T: Send + Sync>() {}
    #[allow(dead_code)]
    fn _check() {
        _assert_send_sync::<Workflow>();
        _assert_send_sync::<WorkflowInstance>();
        _assert_send_sync::<WorkflowStep>();
    }
}
