//! Example demonstrating Cloudflare Flagship (feature flags) from a Rust Worker.
//!
//! Routes:
//! * `/boolean?flag=<key>`   — evaluate a boolean flag
//! * `/string?flag=<key>`    — evaluate a string flag, optionally with `?userId=<id>` context
//! * `/object?flag=<key>`    — evaluate an object flag into a typed struct
//! * `/details?flag=<key>`   — return the full evaluation details envelope

use serde::{Deserialize, Serialize};
use worker::{event, Env, EvaluationContext, Request, Response, Result, RouteContext, Router, Url};

const BINDING: &str = "FLAGS";

#[derive(Serialize, Deserialize)]
struct Theme {
    primary: String,
    secondary: String,
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    Router::new()
        .get_async("/boolean", boolean)
        .get_async("/string", string)
        .get_async("/object", object)
        .get_async("/details", details)
        .run(req, env)
        .await
}

async fn boolean(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let env = ctx.env;
    let url = req.url()?;
    let flag = query(&url, "flag").unwrap_or_else(|| "example-bool".into());
    let value: bool = env
        .flagship(BINDING)?
        .get_boolean_value(&flag, false)
        .await?
        .value_of();
    Response::from_json(&serde_json::json!({ "flag": flag, "value": value }))
}

async fn string(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let env = ctx.env;
    let url = req.url()?;
    let flag = query(&url, "flag").unwrap_or_else(|| "checkout-flow".into());
    let flagship = env.flagship(BINDING)?;
    let value = match query(&url, "userId") {
        Some(user_id) => {
            let ctx = EvaluationContext::new()
                .string("userId", &user_id)
                .string("country", "US");
            flagship
                .get_string_value_with_context(&flag, "control", ctx.as_ref())
                .await?
        }
        None => flagship.get_string_value(&flag, "control").await?,
    };
    Response::from_json(&serde_json::json!({ "flag": flag, "value": value }))
}

async fn object(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let env = ctx.env;
    let url = req.url()?;
    let flag = query(&url, "flag").unwrap_or_else(|| "theme".into());
    let default = Theme {
        primary: "#000000".into(),
        secondary: "#ffffff".into(),
    };
    let value: Theme = env
        .flagship(BINDING)?
        .get_object_value(&flag, &default)
        .await?;
    Response::from_json(&serde_json::json!({ "flag": flag, "value": value }))
}

async fn details(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let env = ctx.env;
    let url = req.url()?;
    let flag = query(&url, "flag").unwrap_or_else(|| "checkout-flow".into());
    let details = env
        .flagship(BINDING)?
        .get_string_details(&flag, "control")
        .await?;
    Response::from_json(&details)
}

fn query(url: &Url, key: &str) -> Option<String> {
    url.query_pairs()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.into_owned())
}
