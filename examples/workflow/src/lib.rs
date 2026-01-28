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
pub struct MyWorkflow {
    #[allow(dead_code)]
    env: Env,
}

impl WorkflowEntrypoint for MyWorkflow {
    fn new(_ctx: Context, env: Env) -> Self {
        Self { env }
    }

    async fn run(
        &self,
        event: WorkflowEvent<serde_json::Value>,
        step: WorkflowStep,
    ) -> Result<serde_json::Value> {
        console_log!("Workflow started with instance ID: {}", event.instance_id);

        let params: MyParams =
            serde_json::from_value(event.payload).map_err(|e| Error::RustError(e.to_string()))?;

        let name_for_step1 = params.name.clone();
        let step1_result = step
            .do_("initial-processing", move || async move {
                console_log!("Processing for user: {}", name_for_step1);
                Ok(serde_json::json!({
                    "processed": true,
                    "user": name_for_step1
                }))
            })
            .await?;

        console_log!("Step 1 completed: {:?}", step1_result);

        console_log!("Step 2: Sleeping for 10 seconds...");
        step.sleep("wait-for-processing", "10 seconds").await?;

        let email_for_step3 = params.email.clone();
        let notification_result = step
            .do_with_config(
                "send-notification",
                StepConfig {
                    retries: Some(RetryConfig {
                        limit: 3,
                        delay: "5 seconds".to_string(),
                        backoff: Some(Backoff::Exponential),
                    }),
                    timeout: Some("1 minute".to_string()),
                },
                move || async move {
                    console_log!("Sending notification to: {}", email_for_step3);
                    Ok(serde_json::json!({
                        "notification_sent": true,
                        "email": email_for_step3
                    }))
                },
            )
            .await?;

        console_log!("Step 3 completed: {:?}", notification_result);

        let output = MyOutput {
            message: format!("Workflow completed for {}", params.name),
            steps_completed: 3,
        };

        Ok(serde_json::to_value(output).unwrap())
    }
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let url = req.url()?;
    let path = url.path();
    let workflow = env.workflow("MY_WORKFLOW")?;

    match (req.method(), path) {
        (Method::Post, "/workflow") => {
            let params = MyParams {
                email: "user@example.com".to_string(),
                name: "Test User".to_string(),
            };

            let instance = workflow
                .create(Some(CreateOptions {
                    id: None,
                    params: Some(params),
                    retention: None,
                }))
                .await?;

            Response::from_json(&serde_json::json!({
                "id": instance.id()?,
                "message": "Workflow created"
            }))
        }

        (Method::Get, path) if path.starts_with("/workflow/") => {
            let id = path.trim_start_matches("/workflow/");
            let instance = workflow.get(id).await?;
            let status = instance.status().await?;

            Response::from_json(&serde_json::json!({
                "id": instance.id()?,
                "status": format!("{:?}", status.status),
                "error": status.error,
                "output": status.output
            }))
        }

        (Method::Post, path) if path.starts_with("/workflow/") && path.ends_with("/pause") => {
            let id = path
                .trim_start_matches("/workflow/")
                .trim_end_matches("/pause");
            let instance = workflow.get(id).await?;
            instance.pause().await?;

            Response::from_json(&serde_json::json!({
                "id": instance.id()?,
                "message": "Workflow paused"
            }))
        }

        (Method::Post, path) if path.starts_with("/workflow/") && path.ends_with("/resume") => {
            let id = path
                .trim_start_matches("/workflow/")
                .trim_end_matches("/resume");
            let instance = workflow.get(id).await?;
            instance.resume().await?;

            Response::from_json(&serde_json::json!({
                "id": instance.id()?,
                "message": "Workflow resumed"
            }))
        }

        _ => Response::error("Not Found", 404),
    }
}
