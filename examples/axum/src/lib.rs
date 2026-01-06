use axum::{routing::get, Router};
use tower_service::Service;
use worker::*;

pub mod resources;

use crate::resources::foo::service::FooService;

struct AppState {
    foo_service: FooService
}

fn router() -> Router {
    let kv = env.kv("EXAMPLE")?;
    let foo_service = FooService::new(kv);

    let app_state = AppState {
        foo_service
    };

    Router::new().route("/", get(root).route("/foo", get(foos::api::get))).with_state(app_state)
}

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    _env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    Ok(router().call(req).await?)
}

pub async fn root() -> &'static str {
    "Hello Axum!"
}
