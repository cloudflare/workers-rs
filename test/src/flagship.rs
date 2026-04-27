use crate::SomeSharedData;
use serde::{Deserialize, Serialize};
use worker::{Env, EvaluationContext, EvaluationDetails, Request, Response, Result};

const BINDING: &str = "FLAGS";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Theme {
    primary: String,
    secondary: String,
}

fn default_theme() -> Theme {
    Theme {
        primary: "#000000".to_string(),
        secondary: "#ffffff".to_string(),
    }
}

fn last_segment(req: &Request) -> Result<String> {
    let url = req.url()?;
    Ok(url
        .path_segments()
        .and_then(|mut s| s.next_back().map(str::to_owned))
        .unwrap_or_default())
}

#[worker::send]
pub async fn handle_boolean(req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let flag = last_segment(&req)?;
    let value = env
        .flagship(BINDING)?
        .get_boolean_value(&flag, false, None)
        .await?;
    Response::from_json(&serde_json::json!({ "flag": flag, "value": value }))
}

#[worker::send]
pub async fn handle_string(req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let flag = last_segment(&req)?;
    let value = env
        .flagship(BINDING)?
        .get_string_value(&flag, "fallback", None)
        .await?;
    Response::from_json(&serde_json::json!({ "flag": flag, "value": value }))
}

#[worker::send]
pub async fn handle_number(req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let flag = last_segment(&req)?;
    let value = env
        .flagship(BINDING)?
        .get_number_value(&flag, 0.0, None)
        .await?;
    Response::from_json(&serde_json::json!({ "flag": flag, "value": value }))
}

#[worker::send]
pub async fn handle_object(req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let flag = last_segment(&req)?;
    let value: Theme = env
        .flagship(BINDING)?
        .get_object_value(&flag, &default_theme(), None)
        .await?;
    Response::from_json(&serde_json::json!({ "flag": flag, "value": value }))
}

#[worker::send]
pub async fn handle_get(req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let flag = last_segment(&req)?;
    let value = env
        .flagship(BINDING)?
        .get::<serde_json::Value>(&flag, "fallback", None)
        .await?;
    Response::from_json(&serde_json::json!({ "flag": flag, "value": value }))
}

#[worker::send]
pub async fn handle_context(req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let user_id = last_segment(&req)?;
    let eval_ctx = EvaluationContext::new()
        .string("userId", &user_id)
        .number("age", 30.0)
        .bool("premium", true);
    let value = env
        .flagship(BINDING)?
        .get_string_value("user-branch", "default", Some(&eval_ctx))
        .await?;
    Response::from_json(&serde_json::json!({ "userId": user_id, "value": value }))
}

#[worker::send]
pub async fn handle_boolean_details(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let flag = last_segment(&req)?;
    let details = env
        .flagship(BINDING)?
        .get_boolean_details(&flag, false, None)
        .await?;
    Response::from_json(&details_to_json(&details))
}

#[worker::send]
pub async fn handle_string_details(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let flag = last_segment(&req)?;
    let details = env
        .flagship(BINDING)?
        .get_string_details(&flag, "fallback", None)
        .await?;
    Response::from_json(&details_to_json(&details))
}

#[worker::send]
pub async fn handle_number_details(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let flag = last_segment(&req)?;
    let details = env
        .flagship(BINDING)?
        .get_number_details(&flag, 0.0, None)
        .await?;
    Response::from_json(&details_to_json(&details))
}

#[worker::send]
pub async fn handle_object_details(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let flag = last_segment(&req)?;
    let details = env
        .flagship(BINDING)?
        .get_object_details(&flag, &default_theme(), None)
        .await?;
    Response::from_json(&details_to_json(&details))
}

fn details_to_json<T: Serialize>(details: &EvaluationDetails<T>) -> serde_json::Value {
    serde_json::json!({
        "flagKey": details.flag_key,
        "value": details.value,
        "variant": details.variant,
        "reason": details.reason,
        "errorCode": details.error_code,
        "errorMessage": details.error_message,
    })
}
