use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyParams {
    pub email: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyOutput {
    pub message: String,
    pub steps_completed: u32,
}

#[workflow]
pub struct MyWorkflow;

impl WorkflowEntrypoint for MyWorkflow {
    type Input = MyParams;
    type Output = MyOutput;

    fn new(_ctx: Context, _env: Env) -> Self {
        Self
    }

    async fn run(&self, event: WorkflowEvent<MyParams>, step: WorkflowStep) -> Result<MyOutput> {
        console_log!(
            "Workflow '{}' started with instance ID: {}",
            event.workflow_name,
            event.instance_id
        );

        // When triggered by a cron schedule (configured via `schedules` on the
        // workflow binding in wrangler.toml), `event.schedule` is populated.
        if let Some(schedule) = &event.schedule {
            console_log!("Triggered by cron schedule: {}", schedule.cron);
        }

        let params = event.payload;

        // Step callbacks are async closures. Each `move` closure owns its copy
        // of what it captures, which every retry reuses.
        let email = params.email.clone();
        step.do_with_config(
            "validate-params",
            StepConfig {
                retries: Some(RetryConfig {
                    limit: 3,
                    delay: "1 second".into(),
                    backoff: None,
                }),
                timeout: None,
                sensitive: None,
            },
            async move |ctx| {
                console_log!(
                    "step '{}' attempt {}/{}",
                    ctx.step.name,
                    ctx.attempt,
                    ctx.config.retries.limit
                );
                if !email.contains('@') {
                    return Err(NonRetryableError::new("invalid email address").into());
                }
                Ok(serde_json::json!({ "valid": true }))
            },
        )
        .await?;

        let name = params.name.clone();
        let step1_result = step
            .do_("initial-processing", async move |_ctx| {
                console_log!("Processing for user: {}", name);
                Ok(serde_json::json!({
                    "processed": true,
                    "user": name
                }))
            })
            .await?;

        console_log!("Step 1 completed: {:?}", step1_result);

        console_log!("Step 2: Sleeping for 10 seconds...");
        step.sleep("wait-for-processing", "10 seconds").await?;

        let email = params.email.clone();
        let notification_result = step
            .do_with_config(
                "send-notification",
                StepConfig {
                    retries: Some(RetryConfig {
                        limit: 3,
                        delay: "5 seconds".into(),
                        backoff: Some(Backoff::Exponential),
                    }),
                    timeout: Some("1 minute".into()),
                    sensitive: None,
                },
                async move |_ctx| {
                    console_log!("Sending notification to: {}", email);
                    if js_sys::Math::random() < 0.5 {
                        return Err("notification service temporarily unavailable".into());
                    }
                    Ok(serde_json::json!({
                        "notification_sent": true,
                        "email": email
                    }))
                },
            )
            .await?;

        console_log!("Step 3 completed: {:?}", notification_result);

        // Step 4: saga-style rollback. If a later step fails, the runtime
        // invokes the rollback handler (in reverse order of completion) to undo
        // this step's side effects. The handler receives the step's output, the
        // original step context, and the error that triggered the rollback.
        let reservation = step
            .do_with_rollback(
                "reserve-inventory",
                async |_ctx| {
                    console_log!("Reserving inventory");
                    Ok(serde_json::json!({ "reservation_id": "resv_123" }))
                },
                Rollback::new(
                    async |rollback: WorkflowRollbackContext<serde_json::Value>| {
                        if let Some(output) = &rollback.output {
                            console_log!(
                                "Rolling back '{}' (caused by: {}) — releasing {}",
                                rollback.ctx.step.name,
                                rollback.error.message,
                                output
                            );
                        }
                        Ok(())
                    },
                ),
            )
            .await?;

        console_log!("Step 4 completed: {:?}", reservation);

        Ok(MyOutput {
            message: format!("Workflow completed for {}", params.name),
            steps_completed: 4,
        })
    }
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .post_async("/workflow", |mut req, ctx| async move {
            let params: MyParams = req.json().await?;
            let instance = ctx
                .env
                .workflow("MY_WORKFLOW")?
                .create(CreateOptions {
                    params: Some(params),
                    ..Default::default()
                })
                .await?;

            Response::from_json(&serde_json::json!({
                "id": instance.id(),
                "message": "Workflow created"
            }))
        })
        .get_async("/workflow/:id", |_req, ctx| async move {
            let instance = workflow_instance(&ctx).await?;
            let status = instance.status().await?;

            Response::from_json(&serde_json::json!({
                "id": instance.id(),
                "status": format!("{:?}", status.status),
                "error": status.error,
                "output": status.output
            }))
        })
        .post_async("/workflow/:id/:action", |_req, ctx| async move {
            let instance = workflow_instance(&ctx).await?;
            let message = match ctx.param("action").map(String::as_str) {
                Some("pause") => {
                    instance.pause().await?;
                    "Workflow paused"
                }
                Some("resume") => {
                    instance.resume().await?;
                    "Workflow resumed"
                }
                _ => return Response::error("Not Found", 404),
            };

            Response::from_json(&serde_json::json!({
                "id": instance.id(),
                "message": message
            }))
        })
        .run(req, env)
        .await
}

async fn workflow_instance(ctx: &RouteContext<()>) -> Result<WorkflowInstance> {
    let id = ctx.param("id").ok_or("missing instance id")?;
    ctx.env.workflow("MY_WORKFLOW")?.get(id).await
}
