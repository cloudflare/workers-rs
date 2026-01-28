use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestParams {
    pub value: String,
}

#[workflow]
pub struct TestWorkflow {
    #[allow(dead_code)]
    env: Env,
}

impl WorkflowEntrypoint for TestWorkflow {
    fn new(_ctx: Context, env: Env) -> Self {
        Self { env }
    }

    async fn run(
        &self,
        event: WorkflowEvent<serde_json::Value>,
        step: WorkflowStep,
    ) -> Result<serde_json::Value> {
        let params: TestParams =
            serde_json::from_value(event.payload).map_err(|e| Error::RustError(e.to_string()))?;

        let result = step
            .do_("process", move || async move {
                Ok(serde_json::json!({ "processed": params.value }))
            })
            .await?;

        Ok(result)
    }
}

pub async fn handle_workflow_create(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let workflow = env.workflow("TEST_WORKFLOW")?;
    let params = TestParams {
        value: "hello".to_string(),
    };
    let instance = workflow
        .create(Some(CreateOptions {
            params: Some(params),
            ..Default::default()
        }))
        .await?;

    Response::from_json(&serde_json::json!({ "id": instance.id()? }))
}

pub async fn handle_workflow_status(
    req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let url = req.url()?;
    let path = url.path();
    let id = path.trim_start_matches("/workflow/status/");
    let workflow = env.workflow("TEST_WORKFLOW")?;
    let instance = workflow.get(id).await?;
    let status = instance.status().await?;

    Response::from_json(&serde_json::json!({
        "status": format!("{:?}", status.status),
        "output": status.output,
        "error": status.error,
    }))
}
