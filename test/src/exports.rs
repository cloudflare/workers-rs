use serde::{Deserialize, Serialize};
use serde_json::json;
use worker::{wasm_bindgen::JsCast, Context, Env, Fetcher, Request, Response, Result};

use crate::{HandlerResponse, SomeSharedData};

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
struct Props {
    greeting: String,
}

pub fn loopback(env: Env, ctx: Context) -> Result<HandlerResponse> {
    let props = ctx.props::<Props>()?;
    let response = Response::from_json(&json!({
        "entrypoint": "Loopback",
        "greeting": props.greeting,
        "env": env.var("SOME_VARIABLE")?.to_string(),
    }))?;
    #[cfg(feature = "http")]
    let response = response.into();
    Ok(response)
}

async fn fetch(binding: &Fetcher) -> Result<Response> {
    let response = binding.fetch("https://loopback.internal/", None).await?;
    #[cfg(feature = "http")]
    let response = Response::try_from(response)?;
    Ok(response)
}

#[worker::send]
pub async fn handle_get(_req: Request, _env: Env, data: SomeSharedData) -> Result<Response> {
    let binding = Fetcher::unchecked_from_js(data.context.exports().get("Loopback")?);
    fetch(&binding).await
}

#[worker::send]
pub async fn handle_props(_req: Request, _env: Env, data: SomeSharedData) -> Result<Response> {
    let exports = data.context.exports();
    let first = Fetcher::unchecked_from_js(exports.get_with_props(
        "Loopback",
        &Props {
            greeting: "first".into(),
        },
    )?);
    let second = Fetcher::unchecked_from_js(exports.get_with_props(
        "Loopback",
        &Props {
            greeting: "second".into(),
        },
    )?);
    let mut results = Vec::new();
    for binding in [&first, &second, &first] {
        results.push(fetch(binding).await?.json::<serde_json::Value>().await?);
    }
    Response::from_json(&results)
}
