use crate::{
    alarm, cache, d1, fetch, form, kv, queue, r2, request, service, socket, user, ws,
    SomeSharedData, GLOBAL_STATE,
};
#[cfg(feature = "http")]
use std::convert::TryInto;
use std::sync::atomic::Ordering;

use worker::{console_log, Env, Fetch, Headers, Request, Response, Result};

#[cfg(not(feature = "http"))]
use worker::{RouteContext, Router};

#[cfg(feature = "http")]
use axum::{
    routing::{delete, get, head, options, patch, post, put},
    Extension,
};

/// Rewrites a handler with legacy http types to use axum extractors / response type.
#[cfg(feature = "http")]
macro_rules! handler (
    ($name:path) => {
        |Extension(env): Extension<Env>, Extension(data): Extension<SomeSharedData>, req: axum::extract::Request| async {
            let resp = $name(req.try_into().expect("convert request"), env, data).await.expect("handler result");
            Into::<http::Response<axum::body::Body>>::into(resp)
        }
    }
);

#[cfg(feature = "http")]
macro_rules! handler_sync (
    ($name:path) => {
        |Extension(env): Extension<Env>, Extension(data): Extension<SomeSharedData>, req: axum::extract::Request| async {
            let resp = $name(req.try_into().expect("convert request"), env, data).expect("handler result");
            Into::<http::Response<axum::body::Body>>::into(resp)
        }
    }
);

#[cfg(not(feature = "http"))]
macro_rules! handler (
    ($name:path) => {
        |req: Request, ctx: RouteContext<SomeSharedData>| async {
            $name(req, ctx.env, ctx.data).await
        }
    }
);

#[cfg(not(feature = "http"))]
macro_rules! handler_sync (
    ($name:path) => {
        |req: Request, ctx: RouteContext<SomeSharedData>| {
            $name(req, ctx.env, ctx.data)
        }
    }
);

#[cfg(feature = "http")]
pub fn make_router(data: SomeSharedData, env: Env) -> axum::Router {
    axum::Router::new()
        .route("/request", get(handler_sync!(request::handle_a_request)))
        .route(
            "/async-request",
            get(handler!(request::handle_async_request)),
        )
        .route("/websocket", get(handler!(ws::handle_websocket)))
        .route("/got-close-event", get(handler!(handle_close_event)))
        .route("/ws-client", get(handler!(ws::handle_websocket_client)))
        .route("/test-data", get(handler!(request::handle_test_data)))
        .route("/xor/:num", post(handler!(request::handle_xor)))
        .route("/headers", post(handler!(request::handle_headers)))
        .route("/formdata-name", post(handler!(form::handle_formdata_name)))
        .route("/is-secret", post(handler!(form::handle_is_secret)))
        .route(
            "/formdata-file-size",
            post(handler!(form::handle_formdata_file_size)),
        )
        .route(
            "/formdata-file-size/:hash",
            get(handler!(form::handle_formdata_file_size_hash)),
        )
        .route(
            "/post-file-size",
            post(handler!(request::handle_post_file_size)),
        )
        .route("/user/:id/test", get(handler!(user::handle_user_id_test)))
        .route("/user/:id", get(handler!(user::handle_user_id)))
        .route(
            "/account/:id/zones",
            post(handler!(user::handle_post_account_id_zones)),
        )
        .route(
            "/account/:id/zones",
            get(handler!(user::handle_get_account_id_zones)),
        )
        .route(
            "/async-text-echo",
            post(handler!(request::handle_async_text_echo)),
        )
        .route("/fetch", get(handler!(fetch::handle_fetch)))
        .route("/fetch_json", get(handler!(fetch::handle_fetch_json)))
        .route(
            "/proxy_request/*url",
            get(handler!(fetch::handle_proxy_request)),
        )
        .route("/durable/alarm", get(handler!(alarm::handle_alarm)))
        .route("/durable/:id", get(handler!(alarm::handle_id)))
        .route("/durable/put-raw", get(handler!(alarm::handle_put_raw)))
        .route("/durable/websocket", get(handler!(alarm::handle_websocket)))
        .route("/var", get(handler!(request::handle_var)))
        .route("/secret", get(handler!(request::handle_secret)))
        .route("/kv/:key/:value", post(handler!(kv::handle_post_key_value)))
        .route("/bytes", get(handler!(request::handle_bytes)))
        .route("/api-data", post(handler!(request::handle_api_data)))
        .route(
            "/nonsense-repeat",
            post(handler!(request::handle_nonsense_repeat)),
        )
        .route("/status/:code", get(handler!(request::handle_status)))
        .route("/", put(handler_sync!(respond)))
        .route("/", patch(handler_sync!(respond)))
        .route("/", delete(handler_sync!(respond)))
        .route("/", head(handler_sync!(respond)))
        .route("/async", put(handler!(respond_async)))
        .route("/async", patch(handler!(respond_async)))
        .route("/async", delete(handler!(respond_async)))
        .route("/async", head(handler!(respond_async)))
        .route("/*catchall", options(handler!(handle_options_catchall)))
        .route(
            "/request-init-fetch",
            get(handler!(fetch::handle_request_init_fetch)),
        )
        .route(
            "/request-init-fetch-post",
            get(handler!(fetch::handle_request_init_fetch_post)),
        )
        .route(
            "/cancelled-fetch",
            get(handler!(fetch::handle_cancelled_fetch)),
        )
        .route("/fetch-timeout", get(handler!(fetch::handle_fetch_timeout)))
        .route(
            "/redirect-default",
            get(handler!(request::handle_redirect_default)),
        )
        .route("/redirect-307", get(handler!(request::handle_redirect_307)))
        .route("/now", get(handler!(request::handle_now)))
        .route("/cloned", get(handler!(request::handle_cloned)))
        .route(
            "/cloned-stream",
            get(handler!(request::handle_cloned_stream)),
        )
        .route("/cloned-fetch", get(handler!(fetch::handle_cloned_fetch)))
        .route("/wait/:delay", get(handler!(request::handle_wait_delay)))
        .route(
            "/custom-response-body",
            get(handler!(request::handle_custom_response_body)),
        )
        .route("/init-called", get(handler!(handle_init_called)))
        .route("/cache-example", get(handler!(cache::handle_cache_example)))
        .route(
            "/cache-api/get/:key",
            get(handler!(cache::handle_cache_api_get)),
        )
        .route(
            "/cache-api/put/:key",
            put(handler!(cache::handle_cache_api_put)),
        )
        .route(
            "/cache-api/delete/:key",
            post(handler!(cache::handle_cache_api_delete)),
        )
        .route("/cache-stream", get(handler!(cache::handle_cache_stream)))
        .route(
            "/remote-by-request",
            get(handler!(service::handle_remote_by_request)),
        )
        .route(
            "/remote-by-path",
            get(handler!(service::handle_remote_by_path)),
        )
        .route("/queue/send/:id", post(handler!(queue::handle_queue_send)))
        .route(
            "/queue/send_batch",
            post(handler!(queue::handle_batch_send)),
        )
        .route("/queue", get(handler!(queue::handle_queue)))
        .route("/d1/prepared", get(handler!(d1::prepared_statement)))
        .route("/d1/batch", get(handler!(d1::batch)))
        .route("/d1/dump", get(handler!(d1::dump)))
        .route("/d1/exec", post(handler!(d1::exec)))
        .route("/d1/error", get(handler!(d1::error)))
        .route("/r2/list-empty", get(handler!(r2::list_empty)))
        .route("/r2/list", get(handler!(r2::list)))
        .route("/r2/get-empty", get(handler!(r2::get_empty)))
        .route("/r2/get", get(handler!(r2::get)))
        .route("/r2/put", put(handler!(r2::put)))
        .route("/r2/put-properties", put(handler!(r2::put_properties)))
        .route("/r2/put-multipart", put(handler!(r2::put_multipart)))
        .route("/r2/delete", delete(handler!(r2::delete)))
        .route(
            "/socket/failed",
            get(handler!(socket::handle_socket_failed)),
        )
        .route("/socket/read", get(handler!(socket::handle_socket_read)))
        .fallback(get(handler!(catchall)))
        .layer(Extension(env))
        .layer(Extension(data))
}

#[cfg(not(feature = "http"))]
pub fn make_router<'a>(data: SomeSharedData) -> Router<'a, SomeSharedData> {
    Router::with_data(data)
        .get("/request", handler_sync!(request::handle_a_request)) // can pass a fn pointer to keep routes tidy
        .get_async("/async-request", handler!(request::handle_async_request))
        .get_async("/websocket", handler!(ws::handle_websocket))
        .get_async("/got-close-event", handler!(handle_close_event))
        .get_async("/ws-client", handler!(ws::handle_websocket_client))
        .get_async("/test-data", handler!(request::handle_test_data))
        .post_async("/xor/:num", handler!(request::handle_xor))
        .post_async("/headers", handler!(request::handle_headers))
        .post_async("/formdata-name", handler!(form::handle_formdata_name))
        .post_async("/is-secret", handler!(form::handle_is_secret))
        .post_async(
            "/formdata-file-size",
            handler!(form::handle_formdata_file_size),
        )
        .get_async(
            "/formdata-file-size/:hash",
            handler!(form::handle_formdata_file_size_hash),
        )
        .post_async("/post-file-size", handler!(request::handle_post_file_size))
        .get_async("/user/:id/test", handler!(user::handle_user_id_test))
        .get_async("/user/:id", handler!(user::handle_user_id))
        .post_async(
            "/account/:id/zones",
            handler!(user::handle_post_account_id_zones),
        )
        .get_async(
            "/account/:id/zones",
            handler!(user::handle_get_account_id_zones),
        )
        .post_async(
            "/async-text-echo",
            handler!(request::handle_async_text_echo),
        )
        .get_async("/fetch", handler!(fetch::handle_fetch))
        .get_async("/fetch_json", handler!(fetch::handle_fetch_json))
        .get_async("/proxy_request/*url", handler!(fetch::handle_proxy_request))
        .get_async("/durable/alarm", handler!(alarm::handle_alarm))
        .get_async("/durable/:id", handler!(alarm::handle_id))
        .get_async("/durable/put-raw", handler!(alarm::handle_put_raw))
        .get_async("/durable/websocket", handler!(alarm::handle_websocket))
        .get_async("/secret", handler!(request::handle_secret))
        .get_async("/var", handler!(request::handle_var))
        .post_async("/kv/:key/:value", handler!(kv::handle_post_key_value))
        .get_async("/bytes", handler!(request::handle_bytes))
        .post_async("/api-data", handler!(request::handle_api_data))
        .post_async(
            "/nonsense-repeat",
            handler!(request::handle_nonsense_repeat),
        )
        .get_async("/status/:code", handler!(request::handle_status))
        .put("/", handler_sync!(respond))
        .patch("/", handler_sync!(respond))
        .delete("/", handler_sync!(respond))
        .head("/", handler_sync!(respond))
        .put_async("/async", handler!(respond_async))
        .patch_async("/async", handler!(respond_async))
        .delete_async("/async", handler!(respond_async))
        .head_async("/async", handler!(respond_async))
        .options_async("/*catchall", handler!(handle_options_catchall))
        .get_async(
            "/request-init-fetch",
            handler!(fetch::handle_request_init_fetch),
        )
        .get_async(
            "/request-init-fetch-post",
            handler!(fetch::handle_request_init_fetch_post),
        )
        .get_async("/cancelled-fetch", handler!(fetch::handle_cancelled_fetch))
        .get_async("/fetch-timeout", handler!(fetch::handle_fetch_timeout))
        .get_async(
            "/redirect-default",
            handler!(request::handle_redirect_default),
        )
        .get_async("/redirect-307", handler!(request::handle_redirect_307))
        .get_async("/now", handler!(request::handle_now))
        .get_async("/cloned", handler!(request::handle_cloned))
        .get_async("/cloned-stream", handler!(request::handle_cloned_stream))
        .get_async("/cloned-fetch", handler!(fetch::handle_cloned_fetch))
        .get_async("/wait/:delay", handler!(request::handle_wait_delay))
        .get_async(
            "/custom-response-body",
            handler!(request::handle_custom_response_body),
        )
        .get_async("/init-called", handler!(handle_init_called))
        .get_async("/cache-example", handler!(cache::handle_cache_example))
        .get_async("/cache-api/get/:key", handler!(cache::handle_cache_api_get))
        .put_async("/cache-api/put/:key", handler!(cache::handle_cache_api_put))
        .post_async(
            "/cache-api/delete/:key",
            handler!(cache::handle_cache_api_delete),
        )
        .get_async("/cache-stream", handler!(cache::handle_cache_stream))
        .get_async(
            "/remote-by-request",
            handler!(service::handle_remote_by_request),
        )
        .get_async("/remote-by-path", handler!(service::handle_remote_by_path))
        .post_async("/queue/send/:id", handler!(queue::handle_queue_send))
        .post_async("/queue/send_batch", handler!(queue::handle_batch_send))
        .get_async("/queue", handler!(queue::handle_queue))
        .get_async("/d1/prepared", handler!(d1::prepared_statement))
        .get_async("/d1/batch", handler!(d1::batch))
        .get_async("/d1/dump", handler!(d1::dump))
        .post_async("/d1/exec", handler!(d1::exec))
        .get_async("/d1/error", handler!(d1::error))
        .get_async("/r2/list-empty", handler!(r2::list_empty))
        .get_async("/r2/list", handler!(r2::list))
        .get_async("/r2/get-empty", handler!(r2::get_empty))
        .get_async("/r2/get", handler!(r2::get))
        .put_async("/r2/put", handler!(r2::put))
        .put_async("/r2/put-properties", handler!(r2::put_properties))
        .put_async("/r2/put-multipart", handler!(r2::put_multipart))
        .delete_async("/r2/delete", handler!(r2::delete))
        .get_async("/socket/failed", handler!(socket::handle_socket_failed))
        .get_async("/socket/read", handler!(socket::handle_socket_read))
        .or_else_any_method_async("/*catchall", handler!(catchall))
}

fn respond(req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    Response::ok(format!("Ok: {}", String::from(req.method()))).map(|resp| {
        let mut headers = Headers::new();
        headers.set("x-testing", "123").unwrap();
        resp.with_headers(headers)
    })
}

async fn respond_async(req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    Response::ok(format!("Ok (async): {}", String::from(req.method()))).map(|resp| {
        let mut headers = Headers::new();
        headers.set("x-testing", "123").unwrap();
        resp.with_headers(headers)
    })
}

#[worker::send]
async fn handle_close_event(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let some_namespace_kv = env.kv("SOME_NAMESPACE")?;
    let got_close_event = some_namespace_kv
        .get("got-close-event")
        .text()
        .await?
        .unwrap_or_else(|| "false".into());

    // Let the integration tests have some way of knowing if we successfully received the closed event.
    Response::ok(got_close_event)
}

#[worker::send]
async fn catchall(req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let uri = req.url()?;
    let path = uri.path();
    console_log!("[or_else_any_method_async] caught: {}", path);

    Fetch::Url("https://github.com/404".parse().unwrap())
        .send()
        .await
        .map(|resp| resp.with_status(404))
}

async fn handle_options_catchall(
    req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let uri = req.url()?;
    let path = uri.path();
    Response::ok(path)
}

async fn handle_init_called(_req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let init_called = GLOBAL_STATE.load(Ordering::SeqCst);
    Response::ok(init_called.to_string())
}
