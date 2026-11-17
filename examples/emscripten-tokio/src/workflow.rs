//! Tokio inside a Workflow. Like an `#[event]` handler, `run`, each step
//! attempt and each rollback call runs on its own Tokio event loop, so steps
//! can spawn tasks and use Tokio timers and I/O.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use worker::*;

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Params {
    /// Fail after the `reserve` step, so that its rollback runs.
    fail: bool,
}

#[workflow]
pub struct TokioWorkflow {
    env: Env,
}

impl WorkflowEntrypoint for TokioWorkflow {
    type Input = Params;
    type Output = String;

    fn new(_ctx: Context, env: Env) -> Self {
        Self { env }
    }

    async fn run(&self, event: WorkflowEvent<Params>, step: WorkflowStep) -> Result<String> {
        let squares: Vec<u32> = step
            .do_("squares", async |_ctx| {
                let handles: Vec<_> = (1..=3u32)
                    .map(|i| {
                        tokio::spawn(async move {
                            sleep(Duration::from_millis(10)).await;
                            i * i
                        })
                    })
                    .collect();
                let mut squares = Vec::new();
                for handle in handles {
                    squares.push(handle.await.map_err(|e| Error::RustError(e.to_string()))?);
                }
                Ok(squares)
            })
            .await?;

        let rollbacks = self.env.kv("ROLLBACKS")?;
        let key = event.instance_id;
        step.do_with_rollback(
            "reserve",
            async |_ctx| {
                sleep(Duration::from_millis(10)).await;
                Ok(())
            },
            Rollback::new(async move |_rollback: WorkflowRollbackContext<()>| {
                sleep(Duration::from_millis(10)).await;
                rollbacks.put(&key, "released")?.execute().await?;
                Ok(())
            }),
        )
        .await?;

        if event.payload.fail {
            step.do_("charge", async |_ctx| {
                Err::<(), Error>(NonRetryableError::new("card declined").into())
            })
            .await?;
        }

        Ok(format!("squares = {squares:?}"))
    }
}

/// `POST /workflow[?fail]`: start an instance and respond with its id.
pub async fn create(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let fail = req.url()?.query_pairs().any(|(k, _)| k == "fail");
    let instance = ctx
        .env
        .workflow("TOKIO_WORKFLOW")?
        .create(CreateOptions {
            params: Some(Params { fail }),
            ..Default::default()
        })
        .await?;
    Response::ok(instance.id())
}

/// `GET /workflow/:id`: the instance's status, with its output or error.
pub async fn status(_: Request, ctx: RouteContext<()>) -> Result<Response> {
    let id = ctx.param("id").unwrap();
    let status = ctx
        .env
        .workflow("TOKIO_WORKFLOW")?
        .get(id)
        .await?
        .status()
        .await?;
    let detail = match (status.output, status.error) {
        (Some(output), _) => output.to_string(),
        (_, Some(error)) => error.message,
        _ => String::new(),
    };
    Response::ok(format!("{:?}: {detail}", status.status))
}

/// `GET /workflow/:id/rollback`: what the `reserve` step's rollback recorded.
pub async fn rollback(_: Request, ctx: RouteContext<()>) -> Result<Response> {
    let id = ctx.param("id").unwrap();
    match ctx.env.kv("ROLLBACKS")?.get(id).text().await? {
        Some(marker) => Response::ok(marker),
        None => Response::error("not rolled back", 404),
    }
}
