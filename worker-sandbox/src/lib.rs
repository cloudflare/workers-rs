use std::{
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::Duration,
};

use bytes::Bytes;
use ::http::Method;
use rand::Rng;
use router_service::unsync::Router;
use serde::{Deserialize, Serialize};
use tower::Service;
use uuid::Uuid;
use worker::{
    body::Body,
    http::{HttpClone, RequestRedirect, Response},
    *,
};

mod alarm;
mod counter;
mod d1;
mod r2;
mod test;
mod utils;
use futures_util::{future::Either, StreamExt, TryStreamExt};

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

fn handle_a_request(req: http::Request<body::Body>) -> Response<body::Body> {
    let cf = req.extensions().get::<Cf>().unwrap();

    Response::new(
        format!(
            "req at: {}, located at: {:?}, within: {}",
            req.uri().path(),
            cf.coordinates().unwrap_or_default(),
            cf.region().unwrap_or_else(|| "unknown region".into())
        )
        .into(),
    )
}

async fn test_clone_req(mut req: http::Request<body::Body>) -> body::Bytes {
    // Change version to something non-default
    *req.version_mut() = http::Version::HTTP_3;

    // Store an extension
    struct Ext;
    req.extensions_mut().insert(Ext);
    req.extensions_mut().insert(AbortSignal::abort());

    // Store original values
    let original_method = req.method().clone();
    let original_verion = req.version();
    let original_uri = req.uri().clone();
    let original_headers = req.headers().clone();

    let clone = req.clone();

    // Make sure original values are kept
    assert_eq!(req.method(), original_method);
    assert_eq!(req.version(), original_verion);
    assert_eq!(req.uri(), &original_uri);
    assert_eq!(req.headers(), &original_headers);
    assert_eq!(req.extensions().len(), 4);
    assert!(req.extensions().get::<Cf>().is_some());
    assert!(req.extensions().get::<Ext>().is_some());
    assert!(req.extensions().get::<RequestRedirect>().is_some());
    assert!(req.extensions().get::<AbortSignal>().is_some());

    // Make sure clone is correct
    assert_eq!(clone.method(), req.method());
    assert_eq!(clone.version(), req.version());
    assert_eq!(clone.uri(), req.uri());
    assert_eq!(clone.headers(), req.headers());
    assert_eq!(clone.extensions().len(), 2);
    assert_eq!(
        clone.extensions().get::<RequestRedirect>(),
        req.extensions().get::<RequestRedirect>()
    );
    assert!(clone.extensions().get::<AbortSignal>().is_some());

    // Make sure body is correct
    let body = req.into_body().bytes().await.unwrap();
    let clone_body = clone.into_body().bytes().await.unwrap();
    assert_eq!(body, clone_body);

    body
}

async fn test_clone_res(body: body::Bytes) -> body::Bytes {
    struct Ext;

    let mut res = http::Response::builder()
        .status(http::StatusCode::CREATED)
        .version(http::Version::HTTP_3)
        .header("foo", "bar")
        .extension(Ext)
        .body(body::Body::from(body))
        .unwrap();

    // Store original values
    let original_status = res.status();
    let original_verion = res.version();
    let original_headers = res.headers().clone();

    let clone = res.clone();

    // Make sure original values are kept
    assert_eq!(res.status(), original_status);
    assert_eq!(res.version(), original_verion);
    assert_eq!(res.headers(), &original_headers);
    assert_eq!(res.extensions().len(), 1);
    assert!(res.extensions().get::<Ext>().is_some());

    // Make sure clone is correct
    assert_eq!(clone.status(), res.status());
    assert_eq!(clone.version(), res.version());
    assert_eq!(clone.headers(), res.headers());
    assert!(clone.extensions().is_empty());

    // Make sure body is correct
    let body = res.into_body().bytes().await.unwrap();
    let clone_body = clone.into_body().bytes().await.unwrap();
    assert_eq!(body, clone_body);

    body
}

static GLOBAL_STATE: AtomicBool = AtomicBool::new(false);

static GLOBAL_QUEUE_STATE: Mutex<Vec<QueueBody>> = Mutex::new(Vec::new());

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
pub async fn main(
    req: worker::http::Request<worker::body::Body>,
    env: Env,
    _ctx: worker::Context,
) -> Result<worker::http::Response<worker::body::Body>> {
    let env = Rc::new(env);
    let mut router: Router<Body, Rc<Env>, Error> = Router::with_data(env)
        .get(
            "/request",
            |req, _| async move { Ok(handle_a_request(req)) },
        )
        .get(
            "/body",
            |_, _| async move { Ok(Response::new("body".into())) },
        )
        .get("/status-code", |_, _| async move {
            Response::builder()
                .status(http::StatusCode::IM_A_TEAPOT)
                .body(Body::empty())
                .map_err(|e| Error::RustError(e.to_string()))
        })
        .post("/headers", |req, _| async move {
            let mut headers = req.headers().clone();
            headers.append("Hello", "World!".parse().unwrap());

            let mut res = Response::new("returned your headers to you.".into());
            *res.headers_mut() = headers;
            Ok(res)
        })
        .post("/echo", |req, _| async move {
            Ok(Response::new(req.into_body()))
        })
        .get("/fetch", |_, _| async move {
            let req = http::Request::post("https://example.com").body(()).unwrap();
            let resp = fetch(req).await?;

            Ok(Response::new(
                format!("received response with status code {:?}", resp.status()).into(),
            ))
        })
        .get("/fetch-cancelled", |_, _| async move {
            let controller = AbortController::default();
            let signal = controller.signal();

            let (tx, rx) = futures_channel::oneshot::channel();

            // Spawns a future that'll make our fetch request and not block this function.
            wasm_bindgen_futures::spawn_local(async move {
                let res = fetch(
                    http::Request::get("https://cloudflare.com")
                        .extension(signal)
                        .body(())
                        .unwrap(),
                )
                .await;

                tx.send(res).unwrap();
            });

            // And then we try to abort that fetch as soon as we start it, hopefully before
            // cloudflare.com responds.
            controller.abort();

            let res = rx.await.unwrap();
            Ok(res.unwrap_or_else(|err| Response::new(err.to_string().into())))
        })
        .get("/wait/:delay", |_, ctx| async move {
            let delay: Delay = match ctx.param("delay").unwrap().parse() {
                Ok(delay) => Duration::from_millis(delay).into(),
                Err(_) => {
                    return Response::builder()
                        .status(400)
                        .body("invalid delay".into())
                        .map_err(|e| Error::RustError(e.to_string()))
                }
            };

            // Wait for the delay to pass
            delay.await;

            Ok(Response::new("Waited!".into()))
        })
        .get("/user/:id/test", |_req, ctx| async move {
            if let Some(id) = ctx.param("id") {
                return Ok(Response::new(format!("TEST user id: {id}").into()));
            }

            Response::builder()
                .status(500)
                .body("error".into())
                .map_err(|e| Error::RustError(e.to_string()))
        })
        .get("/user/:id", |_req, ctx| async move {
            let id = ctx.param("id").unwrap();
            let body = serde_json::to_string(&User {
                id: id.to_string(),
                timestamp: Date::now().as_millis(),
                date_from_int: Date::new(DateInit::Millis(1234567890)).to_string(),
                date_from_str: Date::new(DateInit::String(
                    "Wed Jan 14 1980 23:56:07 GMT-0700 (Mountain Standard Time)".into(),
                ))
                .to_string(),
            })?;

            Ok(Response::new(body.into()))
        })
        .post("/account/:id/zones", |_, ctx| async move {
            Ok(Response::new(
                format!(
                    "Create new zone for Account: {}",
                    ctx.param("id").unwrap_or(&"not found")
                )
                .into(),
            ))
        })
        .get("/account/:id/zones", |_, ctx| async move {
            Ok(Response::new(
                format!(
                    "Account id: {}..... You get a zone, you get a zone!",
                    ctx.param("id").unwrap_or(&"not found")
                )
                .into(),
            ))
        })
        .get("/fetch_json", |_req, _ctx| async move {
            let req = http::Request::get("https://jsonplaceholder.typicode.com/todos/1")
                .body(())
                .unwrap();
            let resp = fetch(req).await?;

            let body = resp.into_body().text().await?;
            let data: ApiData = serde_json::from_str(&body)?;

            Ok(Response::new(
                format!(
                    "API Returned user: {} with title: {} and completed: {}",
                    data.user_id, data.title, data.completed
                )
                .into(),
            ))
        })
        .get("/proxy_request", |_req, _| async move {
            let url = "https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Encoding/contributors.txt";
            let req = http::Request::get(url).body(()).unwrap();
            fetch(req).await
        })
        .get("/durable/alarm", |_req, ctx| async move {
            let namespace = ctx.data.durable_object("ALARM")?;
            let stub = namespace.id_from_name("alarm")?.get_stub()?;
            // when calling fetch to a Durable Object, a full URL must be used. Alternatively, a
            // compatibility flag can be provided in wrangler.toml to opt-in to older behavior:
            // https://developers.cloudflare.com/workers/platform/compatibility-dates#durable-object-stubfetch-requires-a-full-url
            stub.fetch_with_str("https://fake-host/alarm").await
        })
        .get("/durable/:id", |_req, ctx| async move {
            let namespace = ctx.data.durable_object("COUNTER")?;
            let stub = namespace.id_from_name("A")?.get_stub()?;
            // when calling fetch to a Durable Object, a full URL must be used. Alternatively, a
            // compatibility flag can be provided in wrangler.toml to opt-in to older behavior:
            // https://developers.cloudflare.com/workers/platform/compatibility-dates#durable-object-stubfetch-requires-a-full-url
            stub.fetch_with_str("https://fake-host/").await
        })
        .get("/secret", |_, ctx| async move {
            let secret = ctx.data.secret("SOME_SECRET")?;
            Ok(Response::new(secret.to_string().into()))
        })
        .get("/var", |_, ctx| async move {
            let var = ctx.data.var("SOME_VARIABLE")?;
            Ok(Response::new(var.to_string().into()))
        })
        .post("/kv/:key/:value", |_req, ctx| async move {
            let kv = ctx.data.kv("SOME_NAMESPACE")?;
            if let Some(key) = ctx.param("key") {
                if let Some(value) = ctx.param("value") {
                    kv.put(key, value)?.execute().await?;
                }
            }

            let list = &kv.list().execute().await?;
            Response::builder()
                .header("content-type", "application/json")
                .body(serde_json::to_string(&list)?.into())
                .map_err(|e| Error::RustError(e.to_string()))
        })
        .post("/api-data", |req, _ctx| async move {
            let data = req.into_body().bytes().await?;
            let mut todo: ApiData = serde_json::from_slice(&data).unwrap();

            unsafe { todo.title.as_mut_vec().reverse() };

            Ok(Response::new(serde_json::to_vec(&todo)?.into()))
        })
        .put("/", |_, _| async move {
            Ok(Response::builder()
                .header("x-testing", "123")
                .body(().into())
                .unwrap())
        })
        .post("/clone", |req, _| async move {
            let body = test_clone_req(req).await;
            let res_body = test_clone_res(body.clone()).await;
            assert_eq!(body, res_body);

            Ok(http::Response::new(body.into()))
        })
        .post("/clone-inner", |req, _| async move {
            let clone = req.clone_inner().unwrap();

            let body = req.into_body().bytes().await.unwrap();
            let clone_body = clone.into_body().bytes().await.unwrap();
            assert_eq!(body, clone_body);

            // Make sure that cloning a non-JS request returns none
            assert!(http::Request::get("https://example.com")
                .body(body::Body::empty())
                .unwrap()
                .clone_inner()
                .is_none());

            Ok(http::Response::new(body.into()))
        })
        .post("/xor/:num", |req, ctx| async move {
            let num: u8 = match ctx.param("num").unwrap().parse() {
                Ok(num) => num,
                Err(_) => return Response::builder()
                    .status(400)
                    .body("invalid byte".into())
                    .map_err(|e| Error::RustError(e.to_string()))
            };

            let xor_stream = req.into_body().into_stream().map_ok(move |buf| {
                let mut vec = buf.to_vec();
                vec.iter_mut().for_each(|x| *x ^= num);
                Bytes::from(vec)
            });

            let body = worker::body::Body::from_stream(xor_stream)?;
            let resp = Response::builder()
                .body(body)
                .unwrap();
            Ok(resp)
        })
        .get("/request-init-fetch", |_, _| async move {
            let init = RequestInit::new();
            let req = http::Request::post("https://cloudflare.com").body(()).unwrap();
            fetch_with_init(req, &init).await
        })
        .get("/request-init-fetch-post", |_, _| async move {
            let mut init = RequestInit::new();
            init.method = Method::POST;

            let req = http::Request::post("https://httpbin.org/post").body(()).unwrap();
            fetch_with_init(req, &init).await
        })
        .get("/cancelled-fetch", |_, _| async move {
            let controller = AbortController::default();
            let signal = controller.signal();

            let (tx, rx) = futures_channel::oneshot::channel();

            // Spawns a future that'll make our fetch request and not block this function.
            wasm_bindgen_futures::spawn_local({
                async move {
                    let req = http::Request::post("https://cloudflare.com").body(()).unwrap();
                    let resp = fetch_with_signal(req, &signal).await;
                    tx.send(resp).unwrap();
                }
            });

            // And then we try to abort that fetch as soon as we start it, hopefully before
            // cloudflare.com responds.
            controller.abort();

            let res = rx.await.unwrap();
            let res = res.unwrap_or_else(|err| {
                let text = err.to_string();
                Response::new(text.into())
            });

            Ok(res)
        })
        .get("/fetch-timeout", |_, _| async move {
            let controller = AbortController::default();
            let signal = controller.signal();

            let fetch_fut = async {
                let req = http::Request::post("https://miniflare.mocks/delay").body(()).unwrap();
                let resp = fetch_with_signal(req, &signal).await?;
                let text = resp.into_body().text().await?;
                Ok::<String, worker::Error>(text)
            };
            let delay_fut = async {
                Delay::from(Duration::from_millis(1)).await;
                controller.abort();
                Ok(Response::new("Cancelled".into()))
            };

            futures_util::pin_mut!(fetch_fut);
            futures_util::pin_mut!(delay_fut);

            match futures_util::future::select(delay_fut, fetch_fut).await {
                Either::Left((res, cancelled_fut)) => {
                    // Ensure that the cancelled future returns an AbortError.
                    match cancelled_fut.await {
                        Err(e) if e.to_string().contains("AbortError") => { /* Yay! It worked, let's do nothing to celebrate */},
                        Err(e) => panic!("Fetch errored with a different error than expected: {:#?}", e),
                        Ok(text) => panic!("Fetch unexpectedly succeeded: {}", text)
                    }

                    res
                },
                Either::Right(_) => panic!("Delay future should have resolved first"),
            }
        })
        .get("/redirect-default", |_, _| async move {
            Ok(Response::builder()
                .status(302)
                .header("Location", "https://example.com/")
                .body(Body::empty()).unwrap())
        })
        .get("/redirect-307", |_, _| async move {
            Ok(Response::builder()
                .status(307)
                .header("Location", "https://example.com/")
                .body(Body::empty()).unwrap())
        })
        .get("/now", |_, _| async move {
            let now = chrono::Utc::now();
            let js_date: Date = now.into();
            Ok(Response::new(js_date.to_string().into()))
        })
        .get("/custom-response-body", |_, _| async move {
            Ok(Response::new(vec![b'h', b'e', b'l', b'l', b'o'].into()))
        })
        .get("/init-called", |_, _| async move {
            let init_called = GLOBAL_STATE.load(Ordering::SeqCst);
            Ok(Response::new(init_called.to_string().into()))
        })
        .get("/cache-example", |req, _| async move {
            //console_log!("url: {}", req.uri().to_string());
            let cache = Cache::default();
            let key = req.uri().to_string();
            if let Some(resp) = cache.get(&key, true).await? {
                //console_log!("Cache HIT!");
                Ok(resp)
            } else {
                //console_log!("Cache MISS!");

                let mut resp = Response::builder()
                    .header("content-type", "application/json")
                    // Cache API respects Cache-Control headers. Setting s-max-age to 10
                    // will limit the response to be in cache for 10 seconds max
                    .header("cache-control", "s-maxage=10")
                    .body(serde_json::json!({ "timestamp": Date::now().as_millis() }).to_string().into())
                    .map_err(|e| Error::RustError(e.to_string()))
                    .unwrap();

                cache.put(key, resp.clone()).await?;
                Ok(resp)
            }
        })
        .get("/cache-api/get/:key", |_req, ctx| async move {
            if let Some(key) = ctx.param("key") {
                let cache = Cache::default();
                if let Some(resp) = cache.get(format!("https://{key}"), true).await? {
                    return Ok(resp);
                } else {
                    return Ok(Response::new("cache miss".into()));
                }
            }

            Response::builder()
                .status(400)
                .body("key missing".into())
                .map_err(|e| Error::RustError(e.to_string()))
        })
        .put("/cache-api/put/:key", |_req, ctx| async move {
            if let Some(key) = ctx.param("key") {
                let cache = Cache::default();

                let mut resp = Response::builder()
                    .header("content-type", "application/json")
                    // Cache API respects Cache-Control headers. Setting s-max-age to 10
                    // will limit the response to be in cache for 10 seconds max
                    .header("cache-control", "s-maxage=10")
                    .body(serde_json::json!({ "timestamp": Date::now().as_millis() }).to_string().into())
                    .map_err(|e| Error::RustError(e.to_string()))
                    .unwrap();

                cache.put(format!("https://{key}"), resp.clone()).await?;
                return Ok(resp);
            }

            Response::builder()
                .status(400)
                .body("key missing".into())
                .map_err(|e| Error::RustError(e.to_string()))
        })
        .post("/cache-api/delete/:key", |_req, ctx| async move {
            if let Some(key) = ctx.param("key") {
                let cache = Cache::default();

                let res = cache.delete(format!("https://{key}"), true).await?;
                return Ok(Response::new(serde_json::to_string(&res)?.into()));
            }

            Response::builder()
                .status(400)
                .body("key missing".into())
                .map_err(|e| Error::RustError(e.to_string()))
        })
        .get("/cache-stream", |req, _| async move {
            //console_log!("url: {}", req.uri().to_string());
            let cache = Cache::default();
            let key = req.uri().to_string();
            if let Some(resp) = cache.get(&key, true).await? {
                //console_log!("Cache HIT!");
                Ok(resp)
            } else {
                //console_log!("Cache MISS!");
                let mut rng = rand::thread_rng();
                let count = rng.gen_range(0..10);
                let stream = futures_util::stream::repeat("Hello, world!\n")
                    .take(count)
                    .then(|text| async move {
                        Delay::from(Duration::from_millis(50)).await;
                        Result::Ok(text.as_bytes().to_vec())
                    });

                let body = worker::body::Body::from_stream(stream)?;

                //console_log!("resp = {:?}", resp);

                let mut resp = Response::builder()
                    // Cache API respects Cache-Control headers. Setting s-max-age to 10
                    // will limit the response to be in cache for 10 seconds max
                    .header("cache-control", "s-maxage=10")
                    .body(body)
                    .unwrap();

                cache.put(key, resp.clone()).await?;
                Ok(resp)
            }
        })
        .get("/remote-by-request", |req, ctx| async move {
            let fetcher = ctx.data.service("remote")?;
            fetcher.fetch_request(req).await
        })
        .get("/remote-by-path", |req, ctx| async move {
            let fetcher = ctx.data.service("remote")?;
            let mut init = RequestInit::new();
            init.with_method(Method::POST);

            fetcher.fetch(req.uri().to_string(), Some(init)).await
        })
        .post("/queue/send/:id", |_req, ctx| async move {
            let id = match ctx.param("id").map(|id| Uuid::try_parse(id).ok()).and_then(|u|u) {
                Some(id) => id,
                None =>  {
                    return Response::builder()
                    .status(400)
                    .body("error".into())
                    .map_err(|_| Error::RustError("Failed to parse id, expected a UUID".into()));
                }
            };
            let my_queue = match ctx.data.queue("my_queue") {
                Ok(queue) => queue,
                Err(err) => {
                    return Response::builder()
                    .status(500)
                    .body("error".into())
                    .map_err(|_| Error::RustError(format!("Failed to get queue: {err:?}")));
                }
            };
            match my_queue.send(&QueueBody {
                id: id.to_string(),
            }).await {
                Ok(_) => {
                    Ok(Response::new("Message sent".into()))
                }
                Err(err) => {
                    Response::builder()
                    .status(500)
                    .body("error".into())
                    .map_err(|e| Error::RustError(format!("Failed to send message to queue: {:?}", e)))
                }
            }
        })
        .get("/queue", |_req, _ctx| async move {
            let guard = GLOBAL_QUEUE_STATE.lock().unwrap();
            let messages: Vec<QueueBody> = guard.clone();
            let json = serde_json::to_string(&messages).unwrap();
            Ok(Response::new(Body::from(json)))
        })
        .get("/r2/list-empty", |_, ctx| async move {
            r2::list_empty(ctx.data.as_ref()).await
        })
        .get("/r2/list", |_, ctx| async move {
            r2::list(ctx.data.as_ref()).await
        })
        .get("/r2/get-empty", |_, ctx| async move {
            r2::get_empty(ctx.data.as_ref()).await
        })
        .get("/r2/get", |_, ctx| async move {
            r2::get(ctx.data.as_ref()).await
        })
        .put("/r2/put", |_, ctx| async move {
            r2::put(ctx.data.as_ref()).await
        })
        .put("/r2/put-properties", |_, ctx| async move {
            r2::put_properties(ctx.data.as_ref()).await
        })
        .put("/r2/put-multipart", |_, ctx| async move {
            r2::put_multipart(ctx.data.as_ref()).await
        })
        .delete("/r2/delete", |_, ctx| async move {
            r2::delete(ctx.data.as_ref()).await
        })
        .any("/*catchall", |_, ctx| async move {
            Ok(Response::builder()
                .status(404)
                .body(ctx.param("catchall").unwrap().to_string().into())
                .unwrap())
        });

    router.call(req).await
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct QueueBody {
    pub id: String,
}

#[event(queue)]
pub async fn queue(message_batch: MessageBatch<QueueBody>, _env: Env, _ctx: Context) -> Result<()> {
    let mut guard = GLOBAL_QUEUE_STATE.lock().unwrap();
    for message in message_batch.messages()? {
        console_log!(
            "Received queue message {:?}, with id {} and timestamp: {}",
            message.body,
            message.id,
            message.timestamp.to_string()
        );
        guard.push(message.body);
    }
    Ok(())
}
