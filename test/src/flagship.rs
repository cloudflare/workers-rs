use crate::SomeSharedData;
use serde::{Deserialize, Serialize};
use worker::wasm_bindgen::JsValue;
use worker::{
    Env, EvaluationContext, EvaluationDetails, FlagshipEvaluationDetails, Request, Response, Result,
};

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
    let value = env.flagship(BINDING)?.get_boolean_value(&flag, false).await?;
    Response::from_json(&serde_json::json!({ "flag": flag, "value": value }))
}

#[worker::send]
pub async fn handle_string(req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let flag = last_segment(&req)?;
    let value = env
        .flagship(BINDING)?
        .get_string_value(&flag, "fallback")
        .await?;
    Response::from_json(&serde_json::json!({ "flag": flag, "value": value }))
}

#[worker::send]
pub async fn handle_number(req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let flag = last_segment(&req)?;
    let value = env.flagship(BINDING)?.get_number_value(&flag, 0.0).await?;
    Response::from_json(&serde_json::json!({ "flag": flag, "value": value }))
}

#[worker::send]
pub async fn handle_object(req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let flag = last_segment(&req)?;
    let value: Theme = env
        .flagship(BINDING)?
        .get_object_value(&flag, &default_theme())
        .await?;
    Response::from_json(&serde_json::json!({ "flag": flag, "value": value }))
}

#[worker::send]
pub async fn handle_get(req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let flag = last_segment(&req)?;
    let default = JsValue::from_str("fallback");
    let raw = env
        .flagship(BINDING)?
        .get_with_default_value(&flag, &default)
        .await?;
    let value: serde_json::Value = serde_wasm_bindgen::from_value(raw)?;
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
        .get_string_value_with_record("user-branch", "default", eval_ctx.as_record())
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
        .get_boolean_details(&flag, false)
        .await?;
    let value = details.value().as_bool();
    Response::from_json(&details_to_json(&details, value))
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
        .get_string_details(&flag, "fallback")
        .await?;
    let value = details.value().as_string();
    Response::from_json(&details_to_json(&details, value))
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
        .get_number_details(&flag, 0.0)
        .await?;
    let value = details.value().as_f64();
    Response::from_json(&details_to_json(&details, value))
}

#[worker::send]
pub async fn handle_object_details(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let flag = last_segment(&req)?;
    let details: EvaluationDetails<Theme> = env
        .flagship(BINDING)?
        .get_object_details(&flag, &default_theme())
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

fn details_to_json<T: Serialize>(
    details: &FlagshipEvaluationDetails,
    value: T,
) -> serde_json::Value {
    serde_json::json!({
        "flagKey": details.flag_key(),
        "value": value,
        "variant": details.variant(),
        "reason": details.reason(),
        "errorCode": details.error_code(),
        "errorMessage": details.error_message(),
    })
}
