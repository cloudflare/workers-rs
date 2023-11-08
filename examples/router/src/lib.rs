use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Debug, Deserialize, Serialize)]
struct GenericResponse {
    status: u16,
    message: String,
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get_async("/foo", handle_get)
        .post_async("/bar", handle_post)
        .delete_async("/baz", handle_delete)
        .run(req, env)
        .await
}

pub async fn handle_get(_: Request, _ctx: RouteContext<()>) -> worker::Result<Response> {
    Response::from_json(&GenericResponse {
        status: 200,
        message: "You reached a GET route!".to_string(),
    })
}

pub async fn handle_post(_: Request, _ctx: RouteContext<()>) -> worker::Result<Response> {
    Response::from_json(&GenericResponse {
        status: 200,
        message: "You reached a POST route!".to_string(),
    })
}

pub async fn handle_delete(_: Request, _ctx: RouteContext<()>) -> worker::Result<Response> {
    Response::from_json(&GenericResponse {
        status: 200,
        message: "You reached a DELETE route!".to_string(),
    })
}
