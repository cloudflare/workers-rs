use worker::{event, Env, Request, Response, Result};
use worker::kv::KvError;

#[event(fetch)]
async fn main(_req: Request, env: Env, _: worker::Context) -> Result<Response> {
    let kv = env.kv("EXAMPLE")?;
    let list_response = kv.list().limit(100).execute().await.map_err(|e| {
        if matches!(e, KvError::InvalidKvStore(_)) {
            panic!("invalid kv store");
        }
        e
    })?;
    Response::from_html(serde_json::to_string_pretty(&list_response)?)
}
