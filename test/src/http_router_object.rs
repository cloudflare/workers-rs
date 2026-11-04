//! A Durable Object whose `fetch` uses `http` types. With the `http` feature it implements
//! `DurableObject<HttpRequest, http::Response<axum::body::Body>>` and serves requests with an
//! `axum::Router` directly; without it, the same routes are matched by hand on the default
//! `Request` / `Response` types so the tests run in both modes.

use std::cell::Cell;
use std::panic::AssertUnwindSafe;
use std::rc::Rc;

#[cfg(not(feature = "http"))]
use worker::DurableObject;
use worker::{durable_object, Env, Request, Response, Result, State};

use crate::SomeSharedData;

#[durable_object]
pub struct HttpRouterObject {
    state: State,
    hits: AssertUnwindSafe<Rc<Cell<u32>>>,
}

#[cfg(feature = "http")]
mod http_impl {
    use super::HttpRouterObject;
    use axum::{
        extract::{Path, State as AxumState},
        http::{HeaderMap, Method, StatusCode},
        routing::{any, post},
        Router,
    };
    use std::cell::Cell;
    use std::panic::AssertUnwindSafe;
    use std::rc::Rc;
    use tower_service::Service;
    use worker::{send::SendWrapper, DurableObject, Env, HttpRequest, Result, State};

    // axum state must be `Send + Sync`; the Workers runtime is single threaded.
    type Hits = SendWrapper<Rc<Cell<u32>>>;

    async fn greet(
        AxumState(hits): AxumState<Hits>,
        Path(name): Path<String>,
        method: Method,
        headers: HeaderMap,
    ) -> (StatusCode, [(&'static str, String); 1], String) {
        hits.set(hits.get() + 1);
        let from = headers
            .get("x-from")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("nobody")
            .to_string();
        (
            StatusCode::CREATED,
            [("x-hits", hits.get().to_string())],
            format!("{method} hello {name} from {from}"),
        )
    }

    async fn echo(body: String) -> String {
        format!("echo: {body}")
    }

    impl DurableObject<HttpRequest, http::Response<axum::body::Body>> for HttpRouterObject {
        fn new(state: State, _env: Env) -> Self {
            Self {
                state,
                hits: AssertUnwindSafe(Rc::new(Cell::new(0))),
            }
        }

        async fn fetch(&self, req: HttpRequest) -> Result<http::Response<axum::body::Body>> {
            let _ = self.state.id();
            let mut router = Router::new()
                .route("/greet/{name}", any(greet))
                .route("/echo", post(echo))
                .with_state(SendWrapper(self.hits.0.clone()));
            Ok(router.call(req).await?)
        }
    }
}

#[cfg(not(feature = "http"))]
impl DurableObject for HttpRouterObject {
    fn new(state: State, _env: Env) -> Self {
        Self {
            state,
            hits: AssertUnwindSafe(Rc::new(Cell::new(0))),
        }
    }

    async fn fetch(&self, mut req: Request) -> Result<Response> {
        let _ = self.state.id();
        let path = req.path();
        if let Some(name) = path.strip_prefix("/greet/") {
            self.hits.set(self.hits.get() + 1);
            let from = req
                .headers()
                .get("x-from")?
                .unwrap_or_else(|| "nobody".to_string());
            let resp = Response::ok(format!("{} hello {name} from {from}", req.method()))?
                .with_status(201);
            resp.headers().set("x-hits", &self.hits.get().to_string())?;
            return Ok(resp);
        }
        if path == "/echo" && req.method() == worker::Method::Post {
            return Response::ok(format!("echo: {}", req.text().await?));
        }
        Response::error("Not Found", 404)
    }
}

/// Forwards the incoming request to `HttpRouterObject` through [`worker::Stub::fetch_request`],
/// stripping the `/durable/http-router` prefix.
#[worker::send]
pub async fn handle_http_router(req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let mut url = req.url()?;
    let path = url
        .path()
        .strip_prefix("/durable/http-router")
        .unwrap_or("/")
        .to_string();
    url.set_path(&path);
    let forwarded = Request::new_with_init(
        url.as_str(),
        worker::RequestInit::new()
            .with_method(req.method())
            .with_headers(req.headers().clone())
            .with_body(req.inner().body().map(Into::into)),
    )?;

    let stub = env
        .durable_object("HTTP_ROUTER")?
        .id_from_name("http-router")?
        .get_stub()?;
    let resp = stub.fetch_request(forwarded).await?;
    // With the `http` feature this is an `http::Response<worker::Body>`.
    #[cfg(feature = "http")]
    let resp = Response::try_from(resp)?;
    Ok(resp)
}
