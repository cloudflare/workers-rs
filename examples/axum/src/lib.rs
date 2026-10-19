use axum::{extract::State, routing::get, Router};
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
    bucket: Arc<Bucket>,
}

fn router(env: Env) -> Router {
    let kv = env.kv("EXAMPLE").unwrap();
    let foo_service = FooService::new(kv);
    let bucket = env.bucket("BUCKET").unwrap();

    let app_state = AppState {
        foo_service: Arc::new(foo_service),
        bucket: Arc::new(bucket),
    };

    Router::new()
        .route("/", get(root))
        .route("/foo", get(foos::api::get))
        .route("/list-bucket", get(list_bucket))
        .with_state(app_state)
}

/// Lists the objects in the bound R2 bucket.
///
/// `Bucket` itself is `Send + Sync`, so it's fine to store in `AppState`, but
/// `Bucket::list().execute()` still awaits a `wasm_bindgen` `JsFuture`
/// internally, whose state isn't `Send`. That makes the handler's `Future`
/// `!Send`, which `axum` rejects since it requires handler futures to be
/// `Send`. Wrapping the handler body with `#[worker::send]` erases that bound
/// by polling the inner future from within a type that's unconditionally
/// (and safely, since Workers are single-threaded) marked `Send`.
///
/// See <https://github.com/cloudflare/workers-rs/issues/485>.
#[worker::send]
async fn list_bucket(State(AppState { bucket, .. }): State<AppState>) -> String {
    let objects = bucket.list().execute().await.unwrap();
    format!("{} object(s) in bucket", objects.objects().len())
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
