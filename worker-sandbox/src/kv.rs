use super::SomeSharedData;
use worker::{Env, Request, Response, Result};

#[worker::send]
pub async fn handle_post_key_value(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let uri = req.url()?;
    let mut segments = uri.path_segments().unwrap();
    let key = segments.nth(1);
    let value = segments.next();
    let kv = env.kv("SOME_NAMESPACE")?;
    if let Some(key) = key {
        if let Some(value) = value {
            kv.put(key, value)?.execute().await?;
        }
    }

    Response::from_json(&kv.list().execute().await?)
}
