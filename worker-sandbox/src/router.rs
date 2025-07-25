use crate::{
    alarm, analytics_engine, assets, auto_response, cache, counter, d1, durable, fetch, form,
    js_snippets, kv, put_raw, queue, r2, request, secret_store, service, socket, sql_counter,
    sql_iterator, user, ws, SomeSharedData, GLOBAL_STATE,
};
#[cfg(feature = "http")]
use std::convert::TryInto;
use std::sync::atomic::Ordering;

#[cfg(feature = "http")]
use worker::send::SendFuture;

use worker::{console_log, Env, Fetch, Request, Response, ResponseBuilder, Result};

#[cfg(not(feature = "http"))]
use worker::{RouteContext, Router};

#[cfg(feature = "http")]
use axum::{
    routing::{delete, get, head, options, patch, post, put},
    Extension,
};

// Transform the argument into the correct form for the router.
// For axum::Router:
// - "*arg" -> "{*arg}"
// - "arg" -> "{arg}"
#[cfg(feature = "http")]
macro_rules! transform_arg (
    ($arg_name:literal) => {
        format!("{{{}}}", $arg_name)
    }
);
// For worker::Router using matchit=0.7. (Note that in matchit=0.8.6
// (<https://github.com/cloudflare/workers-rs/issues/207>), parameter matching
// is much closer to axum::Router so we should be able to remove all of this
// logic if we update.)
// - "*arg" -> "*arg"
// - "arg" -> ":arg"
#[cfg(not(feature = "http"))]
macro_rules! transform_arg (
    ($arg_name:literal) => {
        if $arg_name.starts_with('*') {
            format!("{}", $arg_name)
        } else {
            format!(":{}", $arg_name)
        }
    };
);

// Like `format!`, but apply `transform_arg!` to each argument first.
macro_rules! format_route (
    ($path_format:literal $(, $arg_name:literal)*) => {
        &format!($path_format, $(
            transform_arg!($arg_name)
        ),*)
    };
);

// Rewrites a handler with legacy http types to use axum extractors / response type.
// Returns an async handler unless the macro is called with a second argument 'sync'.
#[cfg(feature = "http")]
macro_rules! handler (
    ($name:path) => {
        |Extension(env): Extension<Env>, Extension(data): Extension<SomeSharedData>, req: axum::extract::Request| async {
            // SAFETY
            // This can only be called from a worker context
            let resp = unsafe { SendFuture::new($name(req.try_into().expect("convert request"), env, data)).await.expect("handler result") };
            Into::<http::Response<axum::body::Body>>::into(resp)
        }
    };
    ($name:path, sync) => {
        |Extension(env): Extension<Env>, Extension(data): Extension<SomeSharedData>, req: axum::extract::Request| async {
            let resp = $name(req.try_into().expect("convert request"), env, data).expect("handler result");
            Into::<http::Response<axum::body::Body>>::into(resp)
        }
    };
);
#[cfg(not(feature = "http"))]
macro_rules! handler (
    ($name:path, sync) => {
        |req: Request, ctx: RouteContext<SomeSharedData>| {
            $name(req, ctx.env, ctx.data)
        }
    };
    ($name:path) => {
        |req: Request, ctx: RouteContext<SomeSharedData>| async {
            $name(req, ctx.env, ctx.data).await
        }
    };
);

// Add a route to the router. (Ideally this would be a postfix macro
// https://github.com/rust-lang/rfcs/pull/2442.)
#[cfg(feature = "http")]
macro_rules! add_route (
    ($obj:ident, $method:ident, sync, $route:expr, $name:path) => {
        let $obj = $obj.route($route, $method(handler!($name, sync)));
    };
    ($obj:ident, $method:ident, $route:expr, $name:path) => {
        let $obj = $obj.route($route, $method(handler!($name)));
    };
);
// Use `paste::item` to create an identifier '<method>_async' since that's
// the format for worker::Router's async methods.
#[cfg(not(feature = "http"))]
macro_rules! add_route (
    ($obj:ident, $method:ident, $route:expr, $name:path) => {
        paste::item! {
            let $obj = $obj.[<$method _ async>]($route, handler!($name));
        }
    };
    ($obj:ident, $method:ident, sync, $route:expr, $name:path) => {
        let $obj = $obj.$method($route, handler!($name, sync));
    };
);

macro_rules! add_routes (
    ($obj:ident) => {
    add_route!($obj, get, sync, "/request", request::handle_a_request);
    add_route!($obj, get, "/analytics-engine", analytics_engine::handle_analytics_event);
    add_route!($obj, get, "/async-request", request::handle_async_request);
    add_route!($obj, get, format_route!("/asset/{}", "name"), assets::handle_asset);
    add_route!($obj, get, "/websocket", ws::handle_websocket);
    add_route!($obj, get, "/got-close-event", handle_close_event);
    add_route!($obj, get, "/ws-client",ws::handle_websocket_client);
    add_route!($obj, get, "/test-data", request::handle_test_data);
    add_route!($obj, post, format_route!("/xor/{}", "num"), request::handle_xor);
    add_route!($obj, post, "/headers", request::handle_headers);
    add_route!($obj, post, "/formdata-name", form::handle_formdata_name);
    add_route!($obj, post, "/is-secret", form::handle_is_secret);
    add_route!($obj, post, "/formdata-file-size", form::handle_formdata_file_size);
    add_route!($obj, get, format_route!("/formdata-file-size/{}", "hash"), form::handle_formdata_file_size_hash);
    add_route!($obj, post, "/post-file-size",request::handle_post_file_size);
    add_route!($obj, get, format_route!("/user/{}/test", "id"), user::handle_user_id_test);
    add_route!($obj, get, format_route!("/user/{}", "id"), user::handle_user_id);
    add_route!($obj, post, format_route!("/account/{}/zones", "id"), user::handle_post_account_id_zones);
    add_route!($obj, get, format_route!("/account/{}/zones","id"), user::handle_get_account_id_zones);
    add_route!($obj, post, "/async-text-echo",  request::handle_async_text_echo);
    add_route!($obj, get, "/fetch",fetch::handle_fetch);
    add_route!($obj, get, "/fetch_json",fetch::handle_fetch_json);
    add_route!($obj, get, format_route!("/proxy_request/{}", "*url") ,fetch::handle_proxy_request);
    add_route!($obj, get, "/durable/alarm", alarm::handle_alarm);
    add_route!($obj, get, format_route!("/durable/{}", "id"), counter::handle_id);
    add_route!($obj, get, "/durable/put-raw", put_raw::handle_put_raw);
    add_route!($obj, get, "/durable/websocket", counter::handle_websocket);
    add_route!($obj, get,  "/secret", request::handle_secret);
    add_route!($obj, get, "/var", request::handle_var);
    add_route!($obj, get, "/object-var", request::handle_object_var);
    add_route!($obj, post, format_route!("/kv/{}/{}", "key", "value"), kv::handle_post_key_value);
    add_route!($obj, get, "/bytes",request::handle_bytes);
    add_route!($obj, post, "/api-data",request::handle_api_data);
    add_route!($obj, post, "/nonsense-repeat", request::handle_nonsense_repeat);
    add_route!($obj, get, format_route!("/status/{}", "code"), request::handle_status);
    add_route!($obj, put, sync, "/",respond);
    add_route!($obj, patch, sync, "/", respond);
    add_route!($obj, delete, sync, "/",respond);
    add_route!($obj, head, sync, "/", respond);
    add_route!($obj, put, "/async", respond_async);
    add_route!($obj, patch, "/async", respond_async);
    add_route!($obj, delete, "/async", respond_async);
    add_route!($obj, head, "/async", respond_async);
    add_route!($obj, options, format_route!("/{}", "*catchall"), handle_options_catchall);
    add_route!($obj, get, "/request-init-fetch", fetch::handle_request_init_fetch);
    add_route!($obj, get, "/request-init-fetch-post", fetch::handle_request_init_fetch_post);
    add_route!($obj, get, "/cancelled-fetch", fetch::handle_cancelled_fetch);
    add_route!($obj, get, "/fetch-timeout", fetch::handle_fetch_timeout);
    add_route!($obj, get, "/redirect-default", request::handle_redirect_default);
    add_route!($obj, get, "/redirect-307", request::handle_redirect_307);
    add_route!($obj, get, "/now", request::handle_now);
    add_route!($obj, get, "/cloned", request::handle_cloned);
    add_route!($obj, get, "/cloned-stream", request::handle_cloned_stream);
    add_route!($obj, get, "/cloned-fetch", fetch::handle_cloned_fetch);
    add_route!($obj, get, "/cloned-response", fetch::handle_cloned_response_attributes);
    add_route!($obj, get, format_route!("/wait/{}", "delay"), request::handle_wait_delay);
    add_route!($obj, get, "/custom-response-body", request::handle_custom_response_body);
    add_route!($obj, get, "/init-called", handle_init_called);
    add_route!($obj, get, "/cache-example", cache::handle_cache_example);
    add_route!($obj, get, format_route!("/cache-api/get/{}", "key"), cache::handle_cache_api_get);
    add_route!($obj, put, format_route!("/cache-api/put/{}", "key"), cache::handle_cache_api_put);
    add_route!($obj, post, format_route!("/cache-api/delete/{}", "key"), cache::handle_cache_api_delete);
    add_route!($obj, get, "/cache-stream", cache::handle_cache_stream);
    add_route!($obj, get, "/remote-by-request", service::handle_remote_by_request);
    add_route!($obj, get, "/remote-by-path", service::handle_remote_by_path);
    add_route!($obj, post, format_route!("/queue/send/{}", "id"), queue::handle_queue_send);
    add_route!($obj, post, "/queue/send_batch", queue::handle_batch_send);
    add_route!($obj, get, "/queue",queue::handle_queue);
    add_route!($obj, get, "/d1/prepared", d1::prepared_statement);
    add_route!($obj, get, "/d1/batch", d1::batch);
    add_route!($obj, get,  "/d1/dump", d1::dump);
    add_route!($obj, post, "/d1/exec", d1::exec);
    add_route!($obj, get, "/d1/error", d1::error);
    add_route!($obj, get, "/d1/jsvalue_null_is_null", d1::jsvalue_null_is_null);
    add_route!($obj, get, "/d1/serialize_optional_none", d1::serialize_optional_none);
    add_route!($obj, get, "/d1/serialize_optional_some", d1::serialize_optional_some);
    add_route!($obj, get, "/d1/deserialize_optional_none", d1::deserialize_optional_none);
    add_route!($obj, get, "/d1/insert_and_retrieve_optional_none", d1::insert_and_retrieve_optional_none);
    add_route!($obj, get, "/d1/insert_and_retrieve_optional_some", d1::insert_and_retrieve_optional_some);
    add_route!($obj, get, "/d1/retrieve_optional_none", d1::retrieve_optional_none);
    add_route!($obj, get, "/d1/retrieve_optional_some", d1::retrieve_optional_some);
    add_route!($obj, get, "/d1/retrive_first_none", d1::retrive_first_none);
    add_route!($obj, get, "/kv/get", kv::get);
    add_route!($obj, get, "/kv/get-not-found", kv::get_not_found);
    add_route!($obj, get, "/kv/list-keys", kv::list_keys);
    add_route!($obj, get, "/kv/put-simple", kv::put_simple);
    add_route!($obj, get, "/kv/put-metadata", kv::put_metadata);
    add_route!($obj, get, "/kv/put-metadata-struct", kv::put_metadata_struct);
    add_route!($obj, get, "/kv/put-expiration", kv::put_expiration);
    add_route!($obj, get, "/r2/list-empty", r2::list_empty);
    add_route!($obj, get, "/r2/list", r2::list);
    add_route!($obj, get,"/r2/get-empty", r2::get_empty);
    add_route!($obj, get, "/r2/get", r2::get);
    add_route!($obj, put,  "/r2/put", r2::put);
    add_route!($obj, put,  "/r2/put-properties", r2::put_properties);
    add_route!($obj, put,  "/r2/put-multipart", r2::put_multipart);
    add_route!($obj, delete, "/r2/delete", r2::delete);
    add_route!($obj, get, "/socket/failed",  socket::handle_socket_failed);
    add_route!($obj, get, "/socket/read",  socket::handle_socket_read);
    add_route!($obj, get, "/durable/auto-response", auto_response::handle_auto_response);
    add_route!($obj, get, "/durable/hello", durable::handle_hello);
    add_route!($obj, get, "/durable/hello-unique", durable::handle_hello_unique);
    add_route!($obj, get, "/durable/storage", durable::handle_storage);
    add_route!($obj, get, "/durable/handle-basic-test", durable::handle_basic_test);
    add_route!($obj, get, "/js_snippets/now", js_snippets::performance_now);
    add_route!($obj, get, "/js_snippets/log", js_snippets::console_log);
    add_route!($obj, get, format_route!("/sql-counter/{}", "*path"), sql_counter::handle_sql_counter);
    add_route!($obj, get, format_route!("/sql-iterator/{}", "*path"), sql_iterator::handle_sql_iterator);
    add_route!($obj, get, "/get-from-secret-store", secret_store::get_from_secret_store);
    add_route!($obj, get, "/get-from-secret-store-missing", secret_store::get_from_secret_store_missing);
});

#[cfg(feature = "http")]
pub fn make_router(data: SomeSharedData, env: Env) -> axum::Router {
    let router = axum::Router::new();
    add_routes!(router);
    router
        .fallback(get(handler!(catchall)))
        .layer(Extension(env))
        .layer(Extension(data))
}

#[cfg(not(feature = "http"))]
pub fn make_router<'a>(data: SomeSharedData) -> Router<'a, SomeSharedData> {
    let router = Router::with_data(data);
    add_routes!(router);
    router.or_else_any_method_async("/*catchall", handler!(catchall))
}

#[allow(clippy::needless_pass_by_value)]
fn respond(req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    ResponseBuilder::new()
        .with_header("x-testing", "123")?
        .ok(format!("Ok: {}", String::from(req.method())))
}

async fn respond_async(req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    ResponseBuilder::new()
        .with_header("x-testing", "123")?
        .ok(format!("Ok (async): {}", String::from(req.method())))
}

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

async fn catchall(req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let uri = req.url()?;
    let path = uri.path();
    console_log!("[or_else_any_method_async] caught: {}", path);

    let (builder, body) = Fetch::Url("https://github.com/404".parse().unwrap())
        .send()
        .await?
        .into_parts();

    Ok(builder.with_status(404).body(body))
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
