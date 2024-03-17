use crate::{
    alarm, cache, d1, fetch, form, kv, queue, r2, request, service, user, ws, SomeSharedData,
    GLOBAL_STATE,
};
#[cfg(feature = "http")]
use std::convert::TryInto;
use std::sync::atomic::Ordering;
use worker::{console_log, Fetch, Headers, Request, Response, Result, RouteContext};

#[cfg(not(feature = "http"))]
use worker::Router;

#[cfg(feature = "http")]
use worker::{Context, Env};

#[cfg(feature = "http")]
use axum::{routing::get, Extension};
#[cfg(feature = "http")]
use std::sync::Arc;

#[cfg(feature = "http")]
use axum_macros::debug_handler;

/// Rewrites a handler with legacy http types to use axum extractors / response type.
#[cfg(feature = "http")]
macro_rules! handler (
    ($name:path) => {
        |Extension(env): Extension<Arc<Env>>, Extension(data): Extension<SomeSharedData>, req: axum::extract::Request| async {
            let resp = $name(req.try_into().unwrap(), Arc::into_inner(env).unwrap(), data).await.unwrap();
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

#[cfg(feature = "http")]
pub fn make_router(data: SomeSharedData, env: Env) -> axum::Router {
    axum::Router::new()
        .route("/request", get(handler!(request::handle_a_request)))
        .route(
            "/async-request",
            get(handler!(request::handle_async_request)),
        )
        .route("/var", get(handler!(request::handle_var)))
        .route("/secret", get(handler!(request::handle_secret)))
        .route("/websocket", get(handler!(ws::handle_websocket)))
        .layer(Extension(Arc::new(env)))
        .layer(Extension(data))
}

#[cfg(not(feature = "http"))]
pub fn make_router<'a>(data: SomeSharedData) -> Router<'a, SomeSharedData> {
    Router::with_data(data)
        .get_async("/request", handler!(request::handle_a_request)) // can pass a fn pointer to keep routes tidy
        .get_async("/async-request", handler!(request::handle_async_request))
        .get_async("/websocket", handler!(ws::handle_websocket))
        .get_async("/got-close-event", handle_close_event)
        .get_async("/ws-client", ws::handle_websocket_client)
        .get_async("/test-data", request::handle_test_data)
        .post_async("/xor/:num", request::handle_xor)
        .post_async("/headers", request::handle_headers)
        .post_async("/formdata-name", form::handle_formdata_name)
        .post_async("/is-secret", form::handle_is_secret)
        .post_async("/formdata-file-size", form::handle_formdata_file_size)
        .get_async(
            "/formdata-file-size/:hash",
            form::handle_formdata_file_size_hash,
        )
        .post_async("/post-file-size", request::handle_post_file_size)
        .get_async("/user/:id/test", user::handle_user_id_test)
        .get_async("/user/:id", user::handle_user_id)
        .post_async("/account/:id/zones", user::handle_post_account_id_zones)
        .get_async("/account/:id/zones", user::handle_get_account_id_zones)
        .post_async("/async-text-echo", request::handle_async_text_echo)
        .get_async("/fetch", fetch::handle_fetch)
        .get_async("/fetch_json", fetch::handle_fetch_json)
        .get_async("/proxy_request/*url", fetch::handle_proxy_request)
        .get_async("/durable/alarm", alarm::handle_alarm)
        .get_async("/durable/:id", alarm::handle_id)
        .get_async("/durable/put-raw", alarm::handle_put_raw)
        .get_async("/secret", handler!(request::handle_secret))
        .get_async("/var", handler!(request::handle_var))
        .post_async("/kv/:key/:value", kv::handle_post_key_value)
        .get_async("/bytes", request::handle_bytes)
        .post_async("/api-data", request::handle_api_data)
        .post_async("/nonsense-repeat", request::handle_nonsense_repeat)
        .get_async("/status/:code", request::handle_status)
        .put("/", respond)
        .patch("/", respond)
        .delete("/", respond)
        .head("/", respond)
        .put_async("/async", respond_async)
        .patch_async("/async", respond_async)
        .delete_async("/async", respond_async)
        .head_async("/async", respond_async)
        .options_async("/*catchall", handle_options_catchall)
        .get_async("/request-init-fetch", fetch::handle_request_init_fetch)
        .get_async(
            "/request-init-fetch-post",
            fetch::handle_request_init_fetch_post,
        )
        .get_async("/cancelled-fetch", fetch::handle_cancelled_fetch)
        .get_async("/fetch-timeout", fetch::handle_fetch_timeout)
        .get_async("/redirect-default", request::handle_redirect_default)
        .get_async("/redirect-307", request::handle_redirect_307)
        .get_async("/now", request::handle_now)
        .get_async("/cloned", request::handle_cloned)
        .get_async("/cloned-stream", request::handle_cloned_stream)
        .get_async("/cloned-fetch", fetch::handle_cloned_fetch)
        .get_async("/wait/:delay", request::handle_wait_delay)
        .get_async(
            "/custom-response-body",
            request::handle_custom_response_body,
        )
        .get_async("/init-called", handle_init_called)
        .get_async("/cache-example", cache::handle_cache_example)
        .get_async("/cache-api/get/:key", cache::handle_cache_api_get)
        .put_async("/cache-api/put/:key", cache::handle_cache_api_put)
        .post_async("/cache-api/delete/:key", cache::handle_cache_api_delete)
        .get_async("/cache-stream", cache::handle_cache_stream)
        .get_async("/remote-by-request", service::handle_remote_by_request)
        .get_async("/remote-by-path", service::handle_remote_by_path)
        .post_async("/queue/send/:id", queue::handle_queue_send)
        .get_async("/queue", queue::handle_queue)
        .get_async("/d1/prepared", d1::prepared_statement)
        .get_async("/d1/batch", d1::batch)
        .get_async("/d1/dump", d1::dump)
        .post_async("/d1/exec", d1::exec)
        .get_async("/d1/error", d1::error)
        .get_async("/r2/list-empty", r2::list_empty)
        .get_async("/r2/list", r2::list)
        .get_async("/r2/get-empty", r2::get_empty)
        .get_async("/r2/get", r2::get)
        .put_async("/r2/put", r2::put)
        .put_async("/r2/put-properties", r2::put_properties)
        .put_async("/r2/put-multipart", r2::put_multipart)
        .delete_async("/r2/delete", r2::delete)
        .or_else_any_method_async("/*catchall", catchall)
}

fn respond<D>(req: Request, _ctx: RouteContext<D>) -> Result<Response> {
    Response::ok(format!("Ok: {}", String::from(req.method()))).map(|resp| {
        let mut headers = Headers::new();
        headers.set("x-testing", "123").unwrap();
        resp.with_headers(headers)
    })
}

async fn respond_async<D>(req: Request, _ctx: RouteContext<D>) -> Result<Response> {
    Response::ok(format!("Ok (async): {}", String::from(req.method()))).map(|resp| {
        let mut headers = Headers::new();
        headers.set("x-testing", "123").unwrap();
        resp.with_headers(headers)
    })
}

async fn handle_close_event(_req: Request, ctx: RouteContext<SomeSharedData>) -> Result<Response> {
    let some_namespace_kv = ctx.kv("SOME_NAMESPACE")?;
    let got_close_event = some_namespace_kv
        .get("got-close-event")
        .text()
        .await?
        .unwrap_or_else(|| "false".into());

    // Let the integration tests have some way of knowing if we successfully received the closed event.
    Response::ok(got_close_event)
}

async fn catchall(_req: Request, ctx: RouteContext<SomeSharedData>) -> Result<Response> {
    console_log!(
        "[or_else_any_method_async] caught: {}",
        ctx.param("catchall").unwrap_or(&"?".to_string())
    );

    Fetch::Url("https://github.com/404".parse().unwrap())
        .send()
        .await
        .map(|resp| resp.with_status(404))
}

async fn handle_options_catchall(
    _req: Request,
    ctx: RouteContext<SomeSharedData>,
) -> Result<Response> {
    Response::ok(ctx.param("catchall").unwrap())
}

async fn handle_init_called(_req: Request, _ctx: RouteContext<SomeSharedData>) -> Result<Response> {
    let init_called = GLOBAL_STATE.load(Ordering::SeqCst);
    Response::ok(init_called.to_string())
}
