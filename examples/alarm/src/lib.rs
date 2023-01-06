use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use blake2::{Blake2b512, Digest};
use futures_util::{future::Either, StreamExt, TryStreamExt};
use rand::Rng;
use serde::{Deserialize, Serialize};
use worker::*;

mod alarm;
mod counter;
mod test;
mod utils;

#[derive(Deserialize, Serialize)]
struct MyData {
    message: String,
    #[serde(default)]
    is: bool,
    #[serde(default)]
    data: Vec<u8>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiData {
    user_id: i32,
    title: String,
    completed: bool,
}

#[derive(Serialize)]
struct User {
    id: String,
    timestamp: u64,
    date_from_int: String,
    date_from_str: String,
}

#[derive(Deserialize, Serialize)]
struct FileSize {
    name: String,
    size: u32,
}

struct SomeSharedData {
    regex: regex::Regex,
}

fn handle_a_request<D>(req: Request, _ctx: RouteContext<D>) -> Result<Response> {
    Response::ok(format!(
        "req at: {}, located at: {:?}, within: {}",
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    ))
}

async fn handle_async_request<D>(req: Request, _ctx: RouteContext<D>) -> Result<Response> {
    Response::ok(format!(
        "[async] req at: {}, located at: {:?}, within: {}",
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    ))
}

static GLOBAL_STATE: AtomicBool = AtomicBool::new(false);

// We're able to specify a start event that is called when the WASM is initialized before any
// requests. This is useful if you have some global state or setup code, like a logger. This is
// only called once for the entire lifetime of the worker.
#[event(start)]
pub fn start() {
    utils::set_panic_hook();

    // Change some global state so we know that we ran our setup function.
    GLOBAL_STATE.store(true, Ordering::SeqCst);
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    let data = SomeSharedData {
        regex: regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap(),
    };

    let router = Router::with_data(data); // if no data is needed, pass `()` or any other valid data

    router
        .get("/request", handle_a_request) // can pass a fn pointer to keep routes tidy
        .get_async("/async-request", handle_async_request)
        .get("/websocket", |_, ctx| {
            // Accept / handle a websocket connection
            let pair = WebSocketPair::new()?;
            let server = pair.server;
            server.accept()?;

            let some_namespace_kv = ctx.kv("SOME_NAMESPACE")?;

            wasm_bindgen_futures::spawn_local(async move {
                let mut event_stream = server.events().expect("could not open stream");

                while let Some(event) = event_stream.next().await {
                    match event.expect("received error in websocket") {
                        WebsocketEvent::Message(msg) => {
                            if let Some(text) = msg.text() {
                                server.send_with_str(text).expect("could not relay text");
                            }
                        }
                        WebsocketEvent::Close(_) => {
                            // Sets a key in a test KV so the integration tests can query if we
                            // actually got the close event. We can't use the shared dat a for this
                            // because miniflare resets that every request.
                            some_namespace_kv
                                .put("got-close-event", "true")
                                .unwrap()
                                .execute()
                                .await
                                .unwrap();
                        }
                    }
                }
            });

            Response::from_websocket(pair.client)
        })
        .get_async("/durable/alarm", |_req, ctx| async move {
            let namespace = ctx.durable_object("ALARM")?;
            let stub = namespace.id_from_name("alarm")?.get_stub()?;
            // when calling fetch to a Durable Object, a full URL must be used. Alternatively, a
            // compatibility flag can be provided in wrangler.toml to opt-in to older behavior:
            // https://developers.cloudflare.com/workers/platform/compatibility-dates#durable-object-stubfetch-requires-a-full-url
            stub.fetch_with_str("https://fake-host/alarm").await
        })
        .or_else_any_method_async("/*catchall", |_, ctx| async move {
            console_log!(
                "[or_else_any_method_async] caught: {}",
                ctx.param("catchall").unwrap_or(&"?".to_string())
            );

            Fetch::Url("https://github.com/404".parse().unwrap())
                .send()
                .await
                .map(|resp| resp.with_status(404))
        })
        .run(req, env)
        .await
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
