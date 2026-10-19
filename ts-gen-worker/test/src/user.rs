use crate::Env;
use serde::Serialize;
use ts_gen_worker::*;

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
    let url = req.url_parsed()?;
    let id = url.path_segments().unwrap().nth(1);
    if let Some(id) = id {
        return Response::builder().ok(format!("TEST user id: {id}"));
    }

    Response::builder().error("Error", 500)
}

pub async fn handle_user_id(req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let url = req.url_parsed()?;
    let id = url.path_segments().unwrap().nth(1);
    if let Some(id) = id {
        return Response::builder().from_json(&User {
            id: id.to_string(),
            timestamp: js_sys::Date::now() as u64,
            date_from_int: js_sys::Date::new_with_value(1_234_567_890.0).to_string(),
            date_from_str: js_sys::Date::new_with_value(
                "Wed Jan 14 1980 23:56:07 GMT-0700 (Mountain Standard Time)",
            )
            .to_string(),
        });
    }

    Response::builder().error("Bad Request", 400)
}

pub async fn handle_post_account_id_zones(
    req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let url = req.url_parsed()?;
    let id = url.path_segments().unwrap().nth(1);
    Response::builder().ok(format!(
        "Create new zone for Account: {}",
        id.unwrap_or("not found")
    ))
}

pub async fn handle_get_account_id_zones(
    req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let url = req.url_parsed()?;
    let id = url.path_segments().unwrap().nth(1);
    Response::builder().ok(format!(
        "Account id: {}..... You get a zone, you get a zone!",
        id.unwrap_or("not found")
    ))
}
