//! Example demonstrating Cloudflare Flagship (feature flags) from a Rust Worker.
//!
//! Routes:
//! * `/boolean?flag=<key>`   — evaluate a boolean flag
//! * `/string?flag=<key>`    — evaluate a string flag, optionally with `?userId=<id>` context
//! * `/object?flag=<key>`    — evaluate an object flag into a typed struct
//! * `/details?flag=<key>`   — return the full `EvaluationDetails` envelope

use serde::{Deserialize, Serialize};
use worker::{event, Env, EvaluationContext, Request, Response, Result, Router, Url};

const BINDING: &str = "FLAGS";

#[derive(Serialize, Deserialize)]
struct Theme {
    primary: String,
    secondary: String,
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    Router::new()
        .get_async(
            "/boolean",
            |req, ctx| async move { boolean(req, ctx.env).await },
        )
        .get_async(
            "/string",
            |req, ctx| async move { string(req, ctx.env).await },
        )
        .get_async(
            "/object",
            |req, ctx| async move { object(req, ctx.env).await },
        )
        .get_async(
            "/details",
            |req, ctx| async move { details(req, ctx.env).await },
        )
        .run(req, env)
        .await
}

async fn boolean(req: Request, env: Env) -> Result<Response> {
    let url = req.url()?;
    let flag = query(&url, "flag").unwrap_or_else(|| "example-bool".into());
    let value = env
        .flagship(BINDING)?
        .get_boolean_value(&flag, false, None)
        .await?;
    Response::from_json(&serde_json::json!({ "flag": flag, "value": value }))
}

async fn string(req: Request, env: Env) -> Result<Response> {
    let url = req.url()?;
    let flag = query(&url, "flag").unwrap_or_else(|| "checkout-flow".into());
    let ctx = query(&url, "userId").map(|user_id| {
        EvaluationContext::new()
            .string("userId", &user_id)
            .string("country", "US")
    });
    let value = env
        .flagship(BINDING)?
        .get_string_value(&flag, "control", ctx.as_ref())
        .await?;
    Response::from_json(&serde_json::json!({ "flag": flag, "value": value }))
}

async fn object(req: Request, env: Env) -> Result<Response> {
    let url = req.url()?;
    let flag = query(&url, "flag").unwrap_or_else(|| "theme".into());
    let default = Theme {
        primary: "#000000".into(),
        secondary: "#ffffff".into(),
    };
    let value: Theme = env
        .flagship(BINDING)?
        .get_object_value(&flag, &default, None)
        .await?;
    Response::from_json(&serde_json::json!({ "flag": flag, "value": value }))
}

async fn details(req: Request, env: Env) -> Result<Response> {
    let url = req.url()?;
    let flag = query(&url, "flag").unwrap_or_else(|| "checkout-flow".into());
    let details = env
        .flagship(BINDING)?
        .get_string_details(&flag, "control", None)
        .await?;
    Response::from_json(&serde_json::json!({
        "flagKey": details.flag_key,
        "value": details.value,
        "variant": details.variant,
        "reason": details.reason,
        "errorCode": details.error_code,
        "errorMessage": details.error_message,
    }))
}

fn query(url: &Url, key: &str) -> Option<String> {
    url.query_pairs()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.into_owned())
}
