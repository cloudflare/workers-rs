use serde::Serialize;
use worker::{Date, DateInit, Env, Request, Response, Result};

use crate::SomeSharedData;

#[derive(Serialize)]
struct User {
    id: String,
    timestamp: u64,
    date_from_int: String,
    date_from_str: String,
}

pub async fn handle_user_id_test(
    req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let url = req.url()?;
    let id = url.path_segments().unwrap().nth(1);
    if let Some(id) = id {
        return Response::ok(format!("TEST user id: {id}"));
    }

    Response::error("Error", 500)
}

pub async fn handle_user_id(req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let url = req.url()?;
    let id = url.path_segments().unwrap().nth(1);
    if let Some(id) = id {
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
    req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let url = req.url()?;
    let id = url.path_segments().unwrap().nth(1);
    Response::ok(format!(
        "Create new zone for Account: {}",
        id.unwrap_or("not found")
    ))
}

pub async fn handle_get_account_id_zones(
    req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let url = req.url()?;
    let id = url.path_segments().unwrap().nth(1);
    Response::ok(format!(
        "Account id: {}..... You get a zone, you get a zone!",
        id.unwrap_or("not found")
    ))
}
