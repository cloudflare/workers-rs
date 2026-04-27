use serde::{Deserialize, Serialize};
use worker::*;

fn last_path_segment(req: &Request) -> Result<String> {
    let url = req.url()?;
    url.path_segments()
        .and_then(|mut s| s.next_back().map(String::from))
        .ok_or_else(|| Error::RustError("missing path segment".into()))
}

async fn get_workflow_instance(
    req: &Request,
    env: &Env,
    binding: &str,
) -> Result<WorkflowInstance> {
    let id = last_path_segment(req)?;
    let workflow = env.workflow(binding)?;
    workflow.get(&id).await
}

async fn create_workflow_no_params(env: &Env, binding: &str) -> Result<Response> {
    let workflow = env.workflow(binding)?;
    let instance = workflow.create_default().await?;
    Response::from_json(&serde_json::json!({ "id": instance.id() }))
}

fn status_response(status: InstanceStatus) -> Result<Response> {
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
pub struct TestWorkflow {
    #[allow(dead_code)]
    env: Env,
}

impl WorkflowEntrypoint for TestWorkflow {
    type Input = TestParams;
    type Output = TestOutput;

    fn new(_ctx: Context, env: Env) -> Self {
        Self { env }
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
                },
                move |ctx| {
                    let value = value_for_validation.clone();
                    async move {
                        if value.is_empty() {
                            return Err(NonRetryableError::new("value must not be empty").into());
                        }
                        Ok(ValidationOutcome {
                            valid: true,
                            attempt: ctx.attempt,
                        })
                    }
                },
            )
            .await?;

        let processed: String = step
            .do_("process", move |_ctx| {
                let value = value.clone();
                async move { Ok(value) }
            })
            .await?;

        Ok(TestOutput {
            processed,
            validation,
        })
    }
}

async fn create_workflow_with_value(env: &Env, value: &str) -> Result<Response> {
    let workflow = env.workflow("TEST_WORKFLOW")?;
    let params = TestParams {
        value: value.to_string(),
    };
    let instance = workflow
        .create(CreateOptions {
            params: Some(params),
            ..Default::default()
        })
        .await?;

    Response::from_json(&serde_json::json!({ "id": instance.id() }))
}

pub async fn handle_workflow_create(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    create_workflow_with_value(&env, "hello").await
}

pub async fn handle_workflow_create_invalid(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    create_workflow_with_value(&env, "").await
}

pub async fn handle_workflow_status(
    req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let instance = get_workflow_instance(&req, &env, "TEST_WORKFLOW").await?;
    let status = instance.status().await?;
    status_response(status)
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
pub struct EventWorkflow {
    #[allow(dead_code)]
    env: Env,
}

impl WorkflowEntrypoint for EventWorkflow {
    type Input = ();
    type Output = ApprovalResult;

    fn new(_ctx: Context, env: Env) -> Self {
        Self { env }
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

pub async fn handle_event_workflow_create(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    create_workflow_no_params(&env, "EVENT_WORKFLOW").await
}

pub async fn handle_event_workflow_send(
    mut req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let instance = get_workflow_instance(&req, &env, "EVENT_WORKFLOW").await?;
    let payload: serde_json::Value = req.json().await?;
    instance.send_event("approval", payload).await?;
    Response::ok("sent")
}

pub async fn handle_event_workflow_status(
    req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let instance = get_workflow_instance(&req, &env, "EVENT_WORKFLOW").await?;
    let status = instance.status().await?;
    status_response(status)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleResult {
    pub completed: bool,
}

#[workflow]
pub struct LifecycleWorkflow {
    #[allow(dead_code)]
    env: Env,
}

impl WorkflowEntrypoint for LifecycleWorkflow {
    type Input = ();
    type Output = LifecycleResult;

    fn new(_ctx: Context, env: Env) -> Self {
        Self { env }
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

pub async fn handle_lifecycle_workflow_create(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    create_workflow_no_params(&env, "LIFECYCLE_WORKFLOW").await
}

pub async fn handle_lifecycle_workflow_status(
    req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let instance = get_workflow_instance(&req, &env, "LIFECYCLE_WORKFLOW").await?;
    let status = instance.status().await?;
    status_response(status)
}

pub async fn handle_lifecycle_workflow_pause(
    req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let instance = get_workflow_instance(&req, &env, "LIFECYCLE_WORKFLOW").await?;
    instance.pause().await?;
    Response::ok("paused")
}

pub async fn handle_lifecycle_workflow_resume(
    req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let instance = get_workflow_instance(&req, &env, "LIFECYCLE_WORKFLOW").await?;
    instance.resume().await?;
    Response::ok("resumed")
}

pub async fn handle_lifecycle_workflow_terminate(
    req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let instance = get_workflow_instance(&req, &env, "LIFECYCLE_WORKFLOW").await?;
    instance.terminate().await?;
    Response::ok("terminated")
}

pub async fn handle_lifecycle_workflow_restart(
    req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let instance = get_workflow_instance(&req, &env, "LIFECYCLE_WORKFLOW").await?;
    instance.restart().await?;
    Response::ok("restarted")
}
