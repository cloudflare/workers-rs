use axum::{routing::get, Router};
use tower_service::Service;
use worker::*;

fn router() -> Router {
    Router::new().route("/", get(root))
}

#[event(fetch)]
async fn fetch(req: HttpRequest, _env: Env, _ctx: Context) -> Result<HttpResponse> {
    console_error_panic_hook::set_once();

    router()
        .call(req)
        .await
        .map(|r| r.map(|b| b.into()))
        .map_err(|e| e.into())
}

pub async fn root() -> &'static str {
    "Hello Axum!"
}
