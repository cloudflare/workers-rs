use super::SomeSharedData;
use worker::{Request, Response, Result, RouteContext};

pub async fn handle_post_key_value(
    _req: Request,
    ctx: RouteContext<SomeSharedData>,
) -> Result<Response> {
    let kv = ctx.kv("SOME_NAMESPACE")?;
    if let Some(key) = ctx.param("key") {
        if let Some(value) = ctx.param("value") {
            kv.put(key, value)?.execute().await?;
        }
    }

    Response::from_json(&kv.list().execute().await?)
}
