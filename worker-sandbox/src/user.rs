use serde::Serialize;
use worker::{Date, DateInit, Request, Response, Result, RouteContext};

use crate::SomeSharedData;

#[derive(Serialize)]
struct User {
    id: String,
    timestamp: u64,
    date_from_int: String,
    date_from_str: String,
}

pub async fn handle_user_id_test(
    _req: Request,
    ctx: RouteContext<SomeSharedData>,
) -> Result<Response> {
    if let Some(id) = ctx.param("id") {
        return Response::ok(format!("TEST user id: {id}"));
    }

    Response::error("Error", 500)
}

pub async fn handle_user_id(_req: Request, ctx: RouteContext<SomeSharedData>) -> Result<Response> {
    if let Some(id) = ctx.param("id") {
        return Response::from_json(&User {
            id: id.to_string(),
            timestamp: Date::now().as_millis(),
            date_from_int: Date::new(DateInit::Millis(1234567890)).to_string(),
            date_from_str: Date::new(DateInit::String(
                "Wed Jan 14 1980 23:56:07 GMT-0700 (Mountain Standard Time)".into(),
            ))
            .to_string(),
        });
    }

    Response::error("Bad Request", 400)
}

pub async fn handle_post_account_id_zones(
    _req: Request,
    ctx: RouteContext<SomeSharedData>,
) -> Result<Response> {
    Response::ok(format!(
        "Create new zone for Account: {}",
        ctx.param("id").unwrap_or(&"not found".into())
    ))
}

pub async fn handle_get_account_id_zones(
    _req: Request,
    ctx: RouteContext<SomeSharedData>,
) -> Result<Response> {
    Response::ok(format!(
        "Account id: {}..... You get a zone, you get a zone!",
        ctx.param("id").unwrap_or(&"not found".into())
    ))
}
