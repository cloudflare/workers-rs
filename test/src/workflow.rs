use serde::{Deserialize, Serialize};
use worker::*;

use crate::SomeSharedData;

// Routes have the shape `/workflow/{kind}/{action}/{id}`.

fn path_segment(req: &Request, index: usize) -> Result<String> {
    req.url()?
        .path_segments()
        .and_then(|mut segments| segments.nth(index))
        .map(String::from)
        .ok_or_else(|| "missing path segment".into())
}

/// The binding for the route's `{kind}`, e.g. `EVENT_WORKFLOW` for `event`.
fn workflow(req: &Request, env: &Env) -> Result<Workflow> {
    env.workflow(&format!(
        "{}_WORKFLOW",
        path_segment(req, 1)?.to_uppercase()
    ))
}

async fn workflow_instance(req: &Request, env: &Env) -> Result<WorkflowInstance> {
    let id = path_segment(req, 3)?;
    workflow(req, env)?.get(&id).await
}

fn created_response(instance: WorkflowInstance) -> Result<Response> {
    Response::from_json(&serde_json::json!({ "id": instance.id() }))
}

pub async fn handle_workflow_create_default(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    created_response(workflow(&req, &env)?.create_default().await?)
}

pub async fn handle_workflow_status(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let status = workflow_instance(&req, &env).await?.status().await?;
    Response::from_json(&serde_json::json!({
        "status": format!("{:?}", status.status),
        "output": status.output,
        "error": status.error,
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestParams {
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationOutcome {
    pub valid: bool,
    pub attempt: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestOutput {
    pub processed: String,
    pub validation: ValidationOutcome,
}

#[workflow]
pub struct TestWorkflow;

impl WorkflowEntrypoint for TestWorkflow {
    type Input = TestParams;
    type Output = TestOutput;

    fn new(_ctx: Context, _env: Env) -> Self {
        Self
    }

    async fn run(
        &self,
        event: WorkflowEvent<TestParams>,
        step: WorkflowStep,
    ) -> Result<TestOutput> {
        let value = event.payload.value;

        let value_for_validation = value.clone();
        let validation: ValidationOutcome = step
            .do_with_config(
                "validate",
                StepConfig {
                    retries: Some(RetryConfig {
                        limit: 2,
                        delay: "1 second".into(),
                        backoff: None,
                    }),
                    timeout: None,
                    sensitive: None,
                },
                async move |ctx| {
                    if value_for_validation.is_empty() {
                        return Err(NonRetryableError::new("value must not be empty").into());
                    }
                    Ok(ValidationOutcome {
                        valid: true,
                        attempt: ctx.attempt,
                    })
                },
            )
            .await?;

        let processed: String = step
            .do_("process", async move |_ctx| Ok(value.clone()))
            .await?;

        Ok(TestOutput {
            processed,
            validation,
        })
    }
}

async fn create_test_workflow(env: &Env, value: &str) -> Result<Response> {
    let instance = env
        .workflow("TEST_WORKFLOW")?
        .create(CreateOptions {
            params: Some(TestParams {
                value: value.to_string(),
            }),
            ..Default::default()
        })
        .await?;
    created_response(instance)
}

pub async fn handle_test_workflow_create(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    create_test_workflow(&env, "hello").await
}

pub async fn handle_test_workflow_create_invalid(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    create_test_workflow(&env, "").await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalEvent {
    pub approved: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalResult {
    pub approved: bool,
    pub reason: String,
    #[serde(rename = "type")]
    pub event_type: String,
}

#[workflow]
pub struct EventWorkflow;

impl WorkflowEntrypoint for EventWorkflow {
    type Input = ();
    type Output = ApprovalResult;

    fn new(_ctx: Context, _env: Env) -> Self {
        Self
    }

    async fn run(&self, _event: WorkflowEvent<()>, step: WorkflowStep) -> Result<ApprovalResult> {
        let event = step
            .wait_for_event::<ApprovalEvent>(
                "wait-for-approval",
                WaitForEventOptions {
                    type_: "approval".to_string(),
                    timeout: Some("30 seconds".into()),
                },
            )
            .await?;

        Ok(ApprovalResult {
            approved: event.payload.approved,
            reason: event.payload.reason,
            event_type: event.type_,
        })
    }
}

#[worker::send]
pub async fn handle_event_workflow_send(
    mut req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let instance = workflow_instance(&req, &env).await?;
    let payload: serde_json::Value = req.json().await?;
    instance.send_event("approval", payload).await?;
    Response::ok("sent")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleResult {
    pub completed: bool,
}

#[workflow]
pub struct LifecycleWorkflow;

impl WorkflowEntrypoint for LifecycleWorkflow {
    type Input = ();
    type Output = LifecycleResult;

    fn new(_ctx: Context, _env: Env) -> Self {
        Self
    }

    async fn run(&self, _event: WorkflowEvent<()>, step: WorkflowStep) -> Result<LifecycleResult> {
        step.sleep(
            "long-sleep",
            WorkflowSleepDuration::new(60, WorkflowDuration::Seconds),
        )
        .await?;
        Ok(LifecycleResult { completed: true })
    }
}

pub async fn handle_lifecycle_workflow_action(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let instance = workflow_instance(&req, &env).await?;
    match path_segment(&req, 2)?.as_str() {
        "pause" => instance.pause().await?,
        "resume" => instance.resume().await?,
        "terminate" => instance.terminate().await?,
        "restart" => instance.restart().await?,
        "restart-from" => {
            instance
                .restart_with_options(RestartOptions {
                    from: Some(RestartFrom {
                        name: "long-sleep".to_string(),
                        count: Some(1),
                        type_: Some(StepType::Sleep),
                    }),
                })
                .await?
        }
        action => return Response::error(format!("unknown action: {action}"), 400),
    }
    Response::ok("ok")
}

// Saga rollback: a first step succeeds with a rollback handler attached, then a
// second step fails non-retryably. The runtime then invokes the rollback, which
// records a marker in KV keyed by the instance id so the test can confirm it ran.
fn rollback_marker_key(instance_id: &str) -> String {
    format!("rollback:{instance_id}")
}

#[workflow]
pub struct RollbackWorkflow {
    env: Env,
}

impl WorkflowEntrypoint for RollbackWorkflow {
    type Input = ();
    type Output = ();

    fn new(_ctx: Context, env: Env) -> Self {
        Self { env }
    }

    async fn run(&self, event: WorkflowEvent<()>, step: WorkflowStep) -> Result<()> {
        let kv = self.env.kv("ROLLBACK_KV")?;
        let key = rollback_marker_key(&event.instance_id);

        step.do_with_rollback(
            "reserve",
            async |_ctx| Ok(serde_json::json!({ "reserved": true })),
            Rollback::new(
                async move |rollback: WorkflowRollbackContext<serde_json::Value>| {
                    kv.put(&key, rollback.error.message)?.execute().await?;
                    Ok(())
                },
            ),
        )
        .await?;

        // Downstream failure triggers the rollback registered above.
        step.do_("explode", async |_ctx| {
            Err::<serde_json::Value, Error>(NonRetryableError::new("kaboom").into())
        })
        .await?;

        Ok(())
    }
}

#[worker::send]
pub async fn handle_rollback_workflow_marker(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let id = path_segment(&req, 3)?;
    let marker = env
        .kv("ROLLBACK_KV")?
        .get(&rollback_marker_key(&id))
        .text()
        .await?;
    Response::from_json(&serde_json::json!({ "marker": marker }))
}
