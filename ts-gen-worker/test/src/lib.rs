#![cfg(any(js_sys_unstable_apis, target_arch = "wasm32"))]

use js_sys::{Object, Promise};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    LazyLock, Mutex,
};
use ts_gen_worker::*;
use wasm_bindgen::prelude::*;

#[allow(clippy::all)]
mod env;
use env::Env;

mod alarm;
mod analytics_engine;
mod assets;
mod auto_response;
mod cache;
mod container;
mod counter;
mod d1;
mod durable;
mod ext;
mod fetch;
mod form;
mod js_snippets;
mod kv;
mod put_raw;
mod queue;
mod r2;
mod rate_limit;
mod request;
mod secret_store;
mod send_email;
mod service;
mod signal;
mod socket;
mod sql;
mod sql_counter;
mod sql_iterator;
mod user;
mod util;
mod ws;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiData {
    user_id: i32,
    title: String,
    completed: bool,
}

#[derive(Clone)]
pub struct SomeSharedData {
    regex: &'static Regex,
}

static GLOBAL_STATE: AtomicBool = AtomicBool::new(false);
static GLOBAL_SECOND_START: AtomicBool = AtomicBool::new(false);

static GLOBAL_QUEUE_STATE: Mutex<Vec<queue::QueueBody>> = Mutex::new(Vec::new());

static DATA_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap());

#[event(start)]
pub fn start() {
    std::panic::set_hook(Box::new(|info| console_error!("{info}")));
    GLOBAL_STATE.store(true, Ordering::SeqCst);
}

#[event(start)]
pub fn second_start() {
    GLOBAL_SECOND_START.store(true, Ordering::SeqCst);
}

fn text(body: &str, status: u16, content_type: &str) -> Result<Response> {
    Response::builder()
        .status(status)
        .header("content-type", content_type)?
        .ok(body)
}

#[cfg(feature = "http")]
type HandlerRequest = HttpRequest;
#[cfg(not(feature = "http"))]
type HandlerRequest = Request;
#[cfg(feature = "http")]
type HandlerResponse = HttpResponse;
#[cfg(not(feature = "http"))]
type HandlerResponse = Response;

/// Under `http` every request and response round-trips through the `http` crate types.
#[event(fetch, respond_with_errors)]
async fn main(req: HandlerRequest, env: Env, ctx: ExecutionContext) -> Result<HandlerResponse> {
    #[cfg(feature = "http")]
    {
        let res = route(req.try_into()?, env, ctx).await?;
        res.try_into()
    }
    #[cfg(not(feature = "http"))]
    route(req, env, ctx).await
}

async fn route(req: Request, env: Env, ctx: ExecutionContext) -> Result<Response> {
    let data = SomeSharedData { regex: &DATA_REGEX };
    let method = req.method();
    let url = req.url_parsed()?;
    let segments: Vec<&str> = url
        .path_segments()
        .map(|s| s.filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();

    match (method.as_str(), segments.as_slice()) {
        ("GET", []) => text("Hello from Rust!", 200, "text/plain"),
        ("GET", ["env"]) => text(&env.greeting(), 200, "text/plain"),
        ("GET", ["headers"]) => {
            let headers = Headers::new()?;
            headers.append("x-multi", "a")?;
            headers.append("x-multi", "b")?;
            headers.set("x-single", "1")?;
            headers.set("x-deleted", "gone")?;
            headers.delete("x-deleted")?;
            let body = format!(
                "multi={} single={} has_deleted={} has_single={} incoming={}",
                headers.get("x-multi")?.unwrap_or_default(),
                headers.get("x-single")?.unwrap_or_default(),
                headers.has("x-deleted")?,
                headers.has("x-single")?,
                req.headers().get("x-incoming")?.unwrap_or_default(),
            );
            Ok(Response::new_with_str_and_init(
                body,
                &ResponseInit::builder().headers(&headers).build(),
            )?)
        }
        ("POST", ["echo"]) => {
            let body = req.text().await?;
            let name = req.headers().get("x-name")?.unwrap_or_default();
            Ok(Response::new_with_str(format!("{name}: {body}"))?)
        }
        ("POST", ["bytes"]) => {
            let buf = req.array_buffer().await?;
            let bytes = buf.to_vec();
            let sum: u32 = bytes.iter().map(|b| *b as u32).sum();
            let headers = Headers::new()?;
            headers.set("x-sum", sum.to_string())?;
            Ok(Response::new_with_buffer_source_and_init(
                &bytes[..],
                &ResponseInit::builder().headers(&headers).build(),
            )?)
        }
        ("POST", ["clone"]) => {
            let copy = req.clone()?;
            let a = req.text().await?;
            let b = copy.text().await?;
            Ok(Response::new_with_str(format!("{a}|{b}"))?)
        }
        ("GET", ["json"]) => {
            let data = Object::from_iter([
                ("method", JsValue::from(req.method())),
                ("n", JsValue::from(42.0)),
            ]);
            Ok(Response::json(&data)?)
        }
        ("GET", ["redirect"]) => Ok(Response::redirect_with_status(
            "https://example.com/elsewhere",
            302,
        )?),
        ("GET", ["fetch-upstream"]) => {
            let upstream = fetch_with_str("https://upstream.test/data").await?;
            let status = upstream.status();
            let ok = upstream.ok();
            let body = upstream.text().await?;
            text(&format!("{status} {ok} {body}"), 200, "text/plain")
        }
        ("POST", ["fetch-init"]) => {
            let body = req.text().await?;
            let headers = Headers::new()?;
            headers.set("x-forwarded", "yes")?;
            let init = RequestInit::builder()
                .method("PUT")
                .headers(&headers)
                .body_with_str(body)
                .build();
            let upstream = fetch_with_str_and_init("https://upstream.test/echo", &init).await?;
            let echoed = upstream.text().await?;
            Ok(Response::new_with_str(echoed)?)
        }
        ("GET", ["wait-until"]) => {
            ctx.wait_until(&Promise::resolve(&JsValue::NULL));
            ctx.pass_through_on_exception();
            Ok(Response::new_with_str("ok")?)
        }
        ("GET", ["log"]) => {
            console.with(|c| c.log(&[JsValue::from("hello"), 1.0.into()]));
            console.with(|c| c.warn(&["warned".into()]));
            Ok(Response::new_with_str("logged")?)
        }
        ("GET", ["stream"]) => {
            let body = req.body().is_some();
            Ok(Response::new_with_str(format!("body={body}"))?)
        }
        ("GET", ["bad-header"]) => {
            let headers = Headers::new()?;
            let outcome = match headers.set("bad name", "x") {
                Err(JsException::TypeError(e)) => {
                    format!("TypeError: {}", e.message())
                }
                Err(other) => format!("other: {other}"),
                Ok(()) => "ok".to_string(),
            };
            text(&outcome, 200, "text/plain")
        }
        ("GET", ["bad-redirect"]) => {
            let outcome = match Response::redirect_with_status("https://example.com/", 200) {
                Err(JsException::RangeError(_)) => "RangeError".to_string(),
                Err(other) => format!("other: {other}"),
                Ok(_) => "ok".to_string(),
            };
            text(&outcome, 200, "text/plain")
        }
        ("GET", ["error-display"]) => {
            let err: Error = match Headers::new()?.set("bad name", "x") {
                Err(e) => e.into(),
                Ok(()) => "unexpected".into(),
            };
            let matched = matches!(err, Error::Js(JsException::TypeError(_)));
            text(&format!("{matched} {err}"), 200, "text/plain")
        }
        ("GET", ["error"]) => Err(Error::Rust("boom".to_string())),
        ("GET", ["message-event-data"]) => {
            let init = MessageEventInit::new_with_str("");
            js_sys::Reflect::set(&init, "data", &JsValue::from(1))?;
            let kind = match MessageEvent::new("message", &init)?.data() {
                DataKind::JsValue(_) => "other",
                _ => "string or binary",
            };
            text(kind, 200, "text/plain")
        }

        ("GET", ["request"]) => request::handle_a_request(req, env, data),
        ("GET", ["analytics-engine"]) => {
            analytics_engine::handle_analytics_event(req, env, data).await
        }
        ("GET", ["async-request"]) => request::handle_async_request(req, env, data).await,
        ("GET", ["asset", _]) => assets::handle_asset(req, env, data).await,
        ("GET", ["websocket"]) => ws::handle_websocket(req, env, data).await,
        ("GET", ["got-close-event"]) => handle_close_event(req, env, data).await,
        ("GET", ["ws-client"]) => ws::handle_websocket_client(req, env, data).await,
        ("GET", ["test-data"]) => request::handle_test_data(req, env, data).await,
        ("POST", ["xor", _]) => request::handle_xor(req, env, data).await,
        ("POST", ["headers"]) => request::handle_headers(req, env, data).await,
        ("POST", ["formdata-name"]) => form::handle_formdata_name(req, env, data).await,
        ("POST", ["is-secret"]) => form::handle_is_secret(req, env, data).await,
        ("POST", ["formdata-file-size"]) => form::handle_formdata_file_size(req, env, data).await,
        ("GET", ["formdata-file-size", _]) => {
            form::handle_formdata_file_size_hash(req, env, data).await
        }
        ("POST", ["post-file-size"]) => request::handle_post_file_size(req, env, data).await,
        ("GET", ["user", _, "test"]) => user::handle_user_id_test(req, env, data).await,
        ("GET", ["user", _]) => user::handle_user_id(req, env, data).await,
        ("POST", ["account", _, "zones"]) => {
            user::handle_post_account_id_zones(req, env, data).await
        }
        ("GET", ["account", _, "zones"]) => user::handle_get_account_id_zones(req, env, data).await,
        ("POST", ["async-text-echo"]) => request::handle_async_text_echo(req, env, data).await,
        ("GET", ["fetch"]) => fetch::handle_fetch(req, env, data).await,
        ("GET", ["fetch_json"]) => fetch::handle_fetch_json(req, env, data).await,
        ("GET", ["proxy_request", ..]) => fetch::handle_proxy_request(req, env, data).await,
        ("GET", ["durable", "alarm"]) => alarm::handle_alarm(req, env, data).await,
        ("GET", ["durable", "put-raw"]) => put_raw::handle_put_raw(req, env, data).await,
        ("GET", ["durable", "websocket"]) => counter::handle_websocket(req, env, data).await,
        ("GET", ["durable", "auto-response"]) => {
            auto_response::handle_auto_response(req, env, data).await
        }
        ("GET", ["durable", "hello"]) => durable::handle_hello(req, env, data).await,
        ("GET", ["durable", "ctor-name"]) => durable::handle_ctor_name(req, env, data).await,
        ("GET", ["durable", "hello-unique"]) => durable::handle_hello_unique(req, env, data).await,
        ("GET", ["durable", "storage"]) => durable::handle_storage(req, env, data).await,
        ("GET", ["durable", "block-concurrency"]) => {
            durable::handle_block_concurrency(req, env, data).await
        }
        ("GET", ["durable", "block-concurrency-errors-as-values"]) => {
            durable::handle_block_concurrency_errors_as_values(req, env, data).await
        }
        ("GET", ["durable", "block-concurrency-reset-count"]) => {
            durable::handle_block_concurrency_reset_count(req, env, data).await
        }
        ("GET", ["durable", "block-concurrency-reset-trigger"]) => {
            durable::handle_block_concurrency_reset_trigger(req, env, data).await
        }
        ("GET", ["durable", "constructor-init"]) => {
            durable::handle_constructor_init(req, env, data).await
        }
        ("GET", ["durable", "handle-basic-test"]) => {
            durable::handle_basic_test(req, env, data).await
        }
        ("GET", ["durable", "get-by-name"]) => durable::handle_get_by_name(req, env, data).await,
        ("GET", ["durable", "get-by-name-with-location-hint"]) => {
            durable::handle_get_by_name_with_location_hint(req, env, data).await
        }
        ("GET", ["durable", _]) => counter::handle_id(req, env, data).await,
        ("GET", ["secret"]) => request::handle_secret(req, env, data).await,
        ("GET", ["var"]) => request::handle_var(req, env, data).await,
        ("GET", ["object-var"]) => request::handle_object_var(req, env, data).await,
        ("POST", ["kv", _, _]) => kv::handle_post_key_value(req, env, data).await,
        ("GET", ["bytes"]) => request::handle_bytes(req, env, data).await,
        ("POST", ["api-data"]) => request::handle_api_data(req, env, data).await,
        ("POST", ["nonsense-repeat"]) => request::handle_nonsense_repeat(req, env, data).await,
        ("GET", ["status", _]) => request::handle_status(req, env, data).await,
        ("PUT" | "PATCH" | "DELETE" | "HEAD", []) => respond(req, env, data),
        ("PUT" | "PATCH" | "DELETE" | "HEAD", ["async"]) => respond_async(req, env, data).await,
        ("OPTIONS", [..]) => handle_options_catchall(req, env, data).await,
        ("GET", ["request-init-fetch"]) => fetch::handle_request_init_fetch(req, env, data).await,
        ("GET", ["request-init-fetch-post"]) => {
            fetch::handle_request_init_fetch_post(req, env, data).await
        }
        ("GET", ["cancelled-fetch"]) => fetch::handle_cancelled_fetch(req, env, data).await,
        ("GET", ["fetch-timeout"]) => fetch::handle_fetch_timeout(req, env, data).await,
        ("GET", ["redirect-default"]) => request::handle_redirect_default(req, env, data).await,
        ("GET", ["redirect-307"]) => request::handle_redirect_307(req, env, data).await,
        ("GET", ["now"]) => request::handle_now(req, env, data).await,
        ("GET", ["cloned"]) => request::handle_cloned(req, env, data).await,
        ("GET", ["cloned-stream"]) => request::handle_cloned_stream(req, env, data).await,
        ("GET", ["cloned-fetch"]) => fetch::handle_cloned_fetch(req, env, data).await,
        ("GET", ["cloned-response"]) => {
            fetch::handle_cloned_response_attributes(req, env, data).await
        }
        ("GET", ["wait", _]) => request::handle_wait_delay(req, env, data).await,
        ("GET", ["custom-response-body"]) => {
            request::handle_custom_response_body(req, env, data).await
        }
        ("GET", ["init-called"]) => handle_init_called(req, env, data).await,
        ("GET", ["cache-example"]) => cache::handle_cache_example(req, env, data).await,
        ("GET", ["cache-api", "get", _]) => cache::handle_cache_api_get(req, env, data).await,
        ("PUT", ["cache-api", "put", _]) => cache::handle_cache_api_put(req, env, data).await,
        ("POST", ["cache-api", "delete", _]) => {
            cache::handle_cache_api_delete(req, env, data).await
        }
        ("GET", ["cache-stream"]) => cache::handle_cache_stream(req, env, data).await,
        ("POST", ["container", "echo"]) | ("GET", ["container", "ws"]) => {
            container::handle_container(req, env, data).await
        }
        ("GET", ["remote-by-request"]) => service::handle_remote_by_request(req, env, data).await,
        ("GET", ["remote-by-path"]) => service::handle_remote_by_path(req, env, data).await,
        ("POST", ["queue", "send", _]) => queue::handle_queue_send(req, env, data).await,
        ("POST", ["queue", "send_batch"]) => queue::handle_batch_send(req, env, data).await,
        ("GET", ["queue"]) => queue::handle_queue(req, env, data).await,
        ("GET", ["d1", "prepared"]) => d1::prepared_statement(req, env, data).await,
        ("GET", ["d1", "batch"]) => d1::batch(req, env, data).await,
        ("GET", ["d1", "session_prepare_and_batch"]) => {
            d1::session_prepare_and_batch(req, env, data).await
        }
        ("GET", ["d1", "session_bookmark_roundtrip"]) => {
            d1::session_bookmark_roundtrip(req, env, data).await
        }
        ("GET", ["d1", "dump"]) => d1::dump(req, env, data).await,
        ("POST", ["d1", "exec"]) => d1::exec(req, env, data).await,
        ("GET", ["d1", "error"]) => d1::error(req, env, data).await,
        ("GET", ["d1", "jsvalue_null_is_null"]) => d1::jsvalue_null_is_null(req, env, data).await,
        ("GET", ["d1", "serialize_optional_none"]) => {
            d1::serialize_optional_none(req, env, data).await
        }
        ("GET", ["d1", "serialize_optional_some"]) => {
            d1::serialize_optional_some(req, env, data).await
        }
        ("GET", ["d1", "deserialize_optional_none"]) => {
            d1::deserialize_optional_none(req, env, data).await
        }
        ("GET", ["d1", "insert_and_retrieve_optional_none"]) => {
            d1::insert_and_retrieve_optional_none(req, env, data).await
        }
        ("GET", ["d1", "insert_and_retrieve_optional_some"]) => {
            d1::insert_and_retrieve_optional_some(req, env, data).await
        }
        ("GET", ["d1", "retrieve_optional_none"]) => {
            d1::retrieve_optional_none(req, env, data).await
        }
        ("GET", ["d1", "retrieve_optional_some"]) => {
            d1::retrieve_optional_some(req, env, data).await
        }
        ("GET", ["d1", "retrive_first_none"]) => d1::retrive_first_none(req, env, data).await,
        ("GET", ["d1", "deserialize_with_type_mismatch"]) => {
            d1::deserialize_with_type_mismatch(req, env, data).await
        }
        ("GET", ["kv", "get"]) => kv::get(req, env, data).await,
        ("GET", ["kv", "get-not-found"]) => kv::get_not_found(req, env, data).await,
        ("GET", ["kv", "list-keys"]) => kv::list_keys(req, env, data).await,
        ("GET", ["kv", "put-simple"]) => kv::put_simple(req, env, data).await,
        ("GET", ["kv", "put-metadata"]) => kv::put_metadata(req, env, data).await,
        ("GET", ["kv", "put-metadata-struct"]) => kv::put_metadata_struct(req, env, data).await,
        ("GET", ["kv", "put-expiration"]) => kv::put_expiration(req, env, data).await,
        ("GET", ["kv", "get-bulk"]) => kv::get_bulk(req, env, data).await,
        ("GET", ["kv", "get-bulk-empty"]) => kv::get_bulk_empty(req, env, data).await,
        ("GET", ["kv", "get-bulk-limit"]) => kv::get_bulk_limit(req, env, data).await,
        ("GET", ["r2", "list-empty"]) => r2::list_empty(req, env, data).await,
        ("GET", ["r2", "list"]) => r2::list(req, env, data).await,
        ("GET", ["r2", "get-empty"]) => r2::get_empty(req, env, data).await,
        ("GET", ["r2", "get"]) => r2::get(req, env, data).await,
        ("PUT", ["r2", "put"]) => r2::put(req, env, data).await,
        ("PUT", ["r2", "put-properties"]) => r2::put_properties(req, env, data).await,
        ("PUT", ["r2", "put-multipart"]) => r2::put_multipart(req, env, data).await,
        ("DELETE", ["r2", "delete"]) => r2::delete(req, env, data).await,
        ("GET", ["socket", "failed"]) => socket::handle_socket_failed(req, env, data).await,
        ("GET", ["socket", "read"]) => socket::handle_socket_read(req, env, data).await,
        ("GET", ["js_snippets", "now"]) => js_snippets::performance_now(req, env, data).await,
        ("GET", ["js_snippets", "log"]) => js_snippets::console_log(req, env, data).await,
        ("GET", ["sql-counter", ..]) => sql_counter::handle_sql_counter(req, env, data).await,
        ("GET", ["sql-iterator", ..]) => sql_iterator::handle_sql_iterator(req, env, data).await,
        ("GET", ["get-from-secret-store"]) => {
            secret_store::get_from_secret_store(req, env, data).await
        }
        ("GET", ["get-from-secret-store-missing"]) => {
            secret_store::get_from_secret_store_missing(req, env, data).await
        }
        ("GET", ["test-panic"]) => handle_test_panic(req, env, data),
        ("GET", ["test-abort"]) => handle_test_abort(req, env, data),
        ("GET", ["test-oom"]) => handle_test_oom(req, env, data),
        ("GET", ["test-js-error"]) => js_snippets::throw_js_error(req, env, data),
        ("GET", ["rate-limit", "check"]) => {
            rate_limit::handle_rate_limit_check(req, env, data).await
        }
        ("GET", ["rate-limit", "key", _]) => {
            rate_limit::handle_rate_limit_with_key(req, env, data).await
        }
        ("GET", ["rate-limit", "bulk-test"]) => {
            rate_limit::handle_rate_limit_bulk_test(req, env, data).await
        }
        ("GET", ["rate-limit", "reset"]) => {
            rate_limit::handle_rate_limit_reset(req, env, data).await
        }
        ("GET", ["send-email"]) => send_email::handle_send_email(req, env, data).await,
        ("GET", ["signal", "poll"]) => signal::handle_signal_poll(req, env, data),
        (_, ["ext", rest @ ..]) => ext::handle(req, env, ctx, rest).await,
        _ => catchall(req, env, data).await,
    }
}

fn respond(req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    Response::builder()
        .header("x-testing", "123")?
        .ok(format!("Ok: {}", req.method()))
}

async fn respond_async(req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    Response::builder()
        .header("x-testing", "123")?
        .ok(format!("Ok (async): {}", req.method()))
}

async fn handle_close_event(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let some_namespace_kv = env.some_namespace();
    let got_close_event = some_namespace_kv
        .get("got-close-event")
        .await?
        .unwrap_or_else(|| "false".into());

    Response::builder().ok(got_close_event)
}

async fn catchall(req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let path = req.path();
    console_log!("[or_else_any_method_async] caught: {}", path);

    let (builder, body) = fetch_with_str("https://github.com/404").await?.into_parts();
    builder.status(404).body(body)
}

async fn handle_options_catchall(
    req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    Response::builder().ok(req.path())
}

async fn handle_init_called(_req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let init_called = GLOBAL_STATE.load(Ordering::SeqCst);
    let second_init_called = GLOBAL_SECOND_START.load(Ordering::SeqCst);
    Response::builder().ok((init_called && second_init_called).to_string())
}

fn handle_test_panic(_req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    panic!("Intentional panic for testing context abort functionality");
}

fn handle_test_abort(_req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    #[cfg(target_arch = "wasm32")]
    core::arch::wasm32::unreachable();

    #[cfg(not(target_arch = "wasm32"))]
    std::process::abort();
}

fn handle_test_oom(_req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let mut vecs = Vec::new();
    loop {
        vecs.push(vec![0u8; 1024 * 1024 * 1024]);
    }
}
