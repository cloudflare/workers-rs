use axum::{routing::get, Router};
use tower_service::Service;
use worker::*;

struct AppState {
    kv: KvStore
}

fn router() -> Router {
    let kv = env.kv("EXAMPLE")?;
    let foo_service =

    let app_state = AppState {
        kv
    };
    Router::new().route("/", get(root).route("/foo", get(foo_api))).with_state(app_state)
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

#[derive(Serialize)]
struct Foo {
    id: String
    msg: String
}

pub async fn foo_api(State(state): State<App>, Path(foo_id): Path(String)) -> Result<axum::http::Response<Foo>> {
    let foo = state.foo_service.get(foo_id).await
}

struct FooService {
    kv: KvStore
}

impl FooService {
    pub async fn get(foo_id: String) -> Option<Foo> {
        if let Some(q) = self.cache.get::<Foo>(&foo_id).await? {
            return Ok(q);
        }
        None
    }
}
