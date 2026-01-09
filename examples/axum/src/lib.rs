use axum::{routing::get, Router};
use std::sync::Arc;
use tower_service::Service;
use worker::*;

pub mod error;
pub mod resources;

use crate::resources::foos::{self, service::FooService};

/// AppState's readonly fields are all `Arc<T>` for safe sharing between threads
#[derive(Clone)]
struct AppState {
    foo_service: Arc<FooService>,
}

fn router(env: Env) -> Router {
    let kv = env.kv("EXAMPLE").unwrap();
    let foo_service = FooService::new(kv);

    let app_state = AppState {
        foo_service: Arc::new(foo_service),
    };

    Router::new()
        .route("/", get(root))
        .route("/foo", get(foos::api::get))
        .with_state(app_state)
}

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    Ok(router(env).call(req).await?)
}

pub async fn root() -> &'static str {
    "Hello Axum!"
}
