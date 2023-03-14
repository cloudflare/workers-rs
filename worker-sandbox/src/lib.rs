use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::Duration,
};

use blake2::{Blake2b512, Digest};
use futures_util::{future::Either, StreamExt, TryStreamExt};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use worker::*;

mod alarm;
mod counter;
mod r2;
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

pub struct SomeSharedData {
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
        .get_async("/got-close-event", |_, ctx| async move {
            let some_namespace_kv = ctx.kv("SOME_NAMESPACE")?;
            let got_close_event = some_namespace_kv
                .get("got-close-event")
                .text()
                .await?
                .unwrap_or_else(|| "false".into());

            // Let the integration tests have some way of knowing if we successfully received the closed event.
            Response::ok(got_close_event)
        })
        .get_async("/ws-client", |_, _| async move {
            let ws = WebSocket::connect("wss://echo.zeb.workers.dev/".parse()?).await?;

            // It's important that we call this before we send our first message, otherwise we will
            // not have any event listeners on the socket to receive the echoed message.
            let mut event_stream = ws.events()?;

            ws.accept()?;
            ws.send_with_str("Hello, world!")?;

            while let Some(event) = event_stream.next().await {
                let event = event?;

                if let WebsocketEvent::Message(msg) = event {
                    if let Some(text) = msg.text() {
                        return Response::ok(text);
                    }
                }
            }

            Response::error("never got a message echoed back :(", 500)
        })
        .get("/test-data", |_, ctx| {
            // just here to test data works
            if ctx.data.regex.is_match("2014-01-01") {
                Response::ok("data ok")
            } else {
                Response::error("bad match", 500)
            }
        })
        .post("/xor/:num", |mut req, ctx| {
            let num: u8 = match ctx.param("num").unwrap().parse() {
                Ok(num) => num,
                Err(_) => return Response::error("invalid byte", 400),
            };

            let xor_stream = req.stream()?.map_ok(move |mut buf| {
                buf.iter_mut().for_each(|x| *x ^= num);
                buf
            });

            Response::from_stream(xor_stream)
        })
        .post("/headers", |req, _ctx| {
            let mut headers: http::HeaderMap = req.headers().into();
            headers.append("Hello", "World!".parse().unwrap());

            Response::ok("returned your headers to you.")
                .map(|res| res.with_headers(headers.into()))
        })
        .post_async("/formdata-name", |mut req, _ctx| async move {
            let form = req.form_data().await?;
            const NAME: &str = "name";
            let bad_request = Response::error("Bad Request", 400);

            if !form.has(NAME) {
                return bad_request;
            }

            let names: Vec<String> = form
                .get_all(NAME)
                .unwrap_or_default()
                .into_iter()
                .map(|entry| match entry {
                    FormEntry::Field(s) => s,
                    FormEntry::File(f) => f.name(),
                })
                .collect();
            if names.len() > 1 {
                return Response::from_json(&serde_json::json!({ "names": names }));
            }

            if let Some(value) = form.get(NAME) {
                match value {
                    FormEntry::Field(v) => Response::from_json(&serde_json::json!({ NAME: v })),
                    _ => bad_request,
                }
            } else {
                bad_request
            }
        })
        .post_async("/is-secret", |mut req, ctx| async move {
            let form = req.form_data().await?;
            if let Some(secret) = form.get("secret") {
                match secret {
                    FormEntry::Field(name) => {
                        let val = ctx.secret(&name)?;
                        return Response::ok(val.to_string());
                    }
                    _ => return Response::error("Bad Request", 400),
                };
            }

            Response::error("Bad Request", 400)
        })
        .post_async("/formdata-file-size", |mut req, ctx| async move {
            let form = req.form_data().await?;

            if let Some(entry) = form.get("file") {
                return match entry {
                    FormEntry::File(file) => {
                        let kv: kv::KvStore = ctx.kv("FILE_SIZES")?;

                        // create a new FileSize record to store
                        let b = file.bytes().await?;
                        let record = FileSize {
                            name: file.name(),
                            size: b.len() as u32,
                        };

                        // hash the file, and use result as the key
                        let mut hasher = Blake2b512::new();
                        hasher.update(b);
                        let hash = hasher.finalize();
                        let key = hex::encode(&hash[..]);

                        // serialize the record and put it into kv
                        let val = serde_json::to_string(&record)?;
                        kv.put(&key, val)?.execute().await?;

                        // list the default number of keys from the namespace
                        Response::from_json(&kv.list().execute().await?.keys)
                    }
                    _ => Response::error("Bad Request", 400),
                };
            }

            Response::error("Bad Request", 400)
        })
        .get_async("/formdata-file-size/:hash", |_, ctx| async move {
            if let Some(hash) = ctx.param("hash") {
                let kv = ctx.kv("FILE_SIZES")?;
                return match kv.get(hash).json::<FileSize>().await? {
                    Some(val) => Response::from_json(&val),
                    None => Response::error("Not Found", 404),
                };
            }

            Response::error("Bad Request", 400)
        })
        .post_async("/post-file-size", |mut req, _| async move {
            let bytes = req.bytes().await?;
            Response::ok(format!("size = {}", bytes.len()))
        })
        .get("/user/:id/test", |_req, ctx| {
            if let Some(id) = ctx.param("id") {
                return Response::ok(format!("TEST user id: {id}"));
            }

            Response::error("Error", 500)
        })
        .get("/user/:id", |_req, ctx| {
            if let Some(id) = ctx.param("id") {
                return Response::from_json(&User {
                    id: id.to_string(),
                    timestamp: Date::now().as_millis(),
                    date_from_int: Date::new(DateInit::Millis(1234567890)).to_string(),
                    date_from_str: Date::new(DateInit::String(
                        "Wed Jan 14 1980 23:56:07 GMT-0700 (Mountain Standard Time)".into(),
                    ))
                    .to_string(),
                });
            }

            Response::error("Bad Request", 400)
        })
        .post("/account/:id/zones", |_, ctx| {
            Response::ok(format!(
                "Create new zone for Account: {}",
                ctx.param("id").unwrap_or(&"not found".into())
            ))
        })
        .get("/account/:id/zones", |_, ctx| {
            Response::ok(format!(
                "Account id: {}..... You get a zone, you get a zone!",
                ctx.param("id").unwrap_or(&"not found".into())
            ))
        })
        .get_async("/async-text-echo", |mut req, _ctx| async move {
            Response::ok(req.text().await?)
        })
        .get_async("/fetch", |_req, _ctx| async move {
            let req = Request::new("https://example.com", Method::Post)?;
            let resp = Fetch::Request(req).send().await?;
            let resp2 = Fetch::Url("https://example.com".parse()?).send().await?;
            Response::ok(format!(
                "received responses with codes {} and {}",
                resp.status_code(),
                resp2.status_code()
            ))
        })
        .get_async("/fetch_json", |_req, _ctx| async move {
            let data: ApiData = Fetch::Url(
                "https://jsonplaceholder.typicode.com/todos/1"
                    .parse()
                    .unwrap(),
            )
            .send()
            .await?
            .json()
            .await?;
            Response::ok(format!(
                "API Returned user: {} with title: {} and completed: {}",
                data.user_id, data.title, data.completed
            ))
        })
        .get_async("/proxy_request/*url", |_req, ctx| async move {
            let url = ctx.param("url").unwrap().strip_prefix('/').unwrap();

            Fetch::Url(url.parse()?).send().await
        })
        .get_async("/durable/alarm", |_req, ctx| async move {
            let namespace = ctx.durable_object("ALARM")?;
            let stub = namespace.id_from_name("alarm")?.get_stub()?;
            // when calling fetch to a Durable Object, a full URL must be used. Alternatively, a
            // compatibility flag can be provided in wrangler.toml to opt-in to older behavior:
            // https://developers.cloudflare.com/workers/platform/compatibility-dates#durable-object-stubfetch-requires-a-full-url
            stub.fetch_with_str("https://fake-host/alarm").await
        })
        .get_async("/durable/:id", |_req, ctx| async move {
            let namespace = ctx.durable_object("COUNTER")?;
            let stub = namespace.id_from_name("A")?.get_stub()?;
            // when calling fetch to a Durable Object, a full URL must be used. Alternatively, a
            // compatibility flag can be provided in wrangler.toml to opt-in to older behavior:
            // https://developers.cloudflare.com/workers/platform/compatibility-dates#durable-object-stubfetch-requires-a-full-url
            stub.fetch_with_str("https://fake-host/").await
        })
        .get("/secret", |_req, ctx| {
            Response::ok(ctx.secret("SOME_SECRET")?.to_string())
        })
        .get("/var", |_req, ctx| {
            Response::ok(ctx.var("SOME_VARIABLE")?.to_string())
        })
        .post_async("/kv/:key/:value", |_req, ctx| async move {
            let kv = ctx.kv("SOME_NAMESPACE")?;
            if let Some(key) = ctx.param("key") {
                if let Some(value) = ctx.param("value") {
                    kv.put(key, value)?.execute().await?;
                }
            }

            Response::from_json(&kv.list().execute().await?)
        })
        .get("/bytes", |_, _| {
            Response::from_bytes(vec![1, 2, 3, 4, 5, 6, 7])
        })
        .post_async("/api-data", |mut req, _ctx| async move {
            let data = req.bytes().await?;
            let mut todo: ApiData = serde_json::from_slice(&data)?;

            unsafe { todo.title.as_mut_vec().reverse() };

            console_log!("todo = (title {}) (id {})", todo.title, todo.user_id);

            Response::from_bytes(serde_json::to_vec(&todo)?)
        })
        .post_async("/nonsense-repeat", |_, ctx| async move {
            if ctx.data.regex.is_match("2014-01-01") {
                Response::ok("data ok")
            } else {
                Response::error("bad match", 500)
            }
        })
        .get("/status/:code", |_, ctx| {
            if let Some(code) = ctx.param("code") {
                return match code.parse::<u16>() {
                    Ok(status) => Response::ok("You set the status code!")
                        .map(|resp| resp.with_status(status)),
                    Err(_e) => Response::error("Failed to parse your status code.", 400),
                };
            }

            Response::error("Bad Request", 400)
        })
        .put("/", respond)
        .patch("/", respond)
        .delete("/", respond)
        .head("/", respond)
        .put_async("/async", respond_async)
        .patch_async("/async", respond_async)
        .delete_async("/async", respond_async)
        .head_async("/async", respond_async)
        .options("/*catchall", |_, ctx| {
            Response::ok(ctx.param("catchall").unwrap())
        })
        .get_async("/request-init-fetch", |_, _| async move {
            let init = RequestInit::new();
            Fetch::Request(Request::new_with_init("https://cloudflare.com", &init)?)
                .send()
                .await
        })
        .get_async("/request-init-fetch-post", |_, _| async move {
            let mut init = RequestInit::new();
            init.method = Method::Post;
            Fetch::Request(Request::new_with_init("https://httpbin.org/post", &init)?)
                .send()
                .await
        })
        .get_async("/cancelled-fetch", |_, _| async move {
            let controller = AbortController::default();
            let signal = controller.signal();

            let (tx, rx) = futures_channel::oneshot::channel();

            // Spawns a future that'll make our fetch request and not block this function.
            wasm_bindgen_futures::spawn_local({
                async move {
                    let fetch = Fetch::Url("https://cloudflare.com".parse().unwrap());
                    let res = fetch.send_with_signal(&signal).await;
                    tx.send(res).unwrap();
                }
            });

            // And then we try to abort that fetch as soon as we start it, hopefully before
            // cloudflare.com responds.
            controller.abort();

            let res = rx.await.unwrap();
            let res = res.unwrap_or_else(|err| {
                let text = err.to_string();
                Response::ok(text).unwrap()
            });

            Ok(res)
        })
        .get_async("/fetch-timeout", |_, _| async move {
            let controller = AbortController::default();
            let signal = controller.signal();

            let fetch_fut = async {
                let fetch = Fetch::Url("http://localhost:8787/wait/10000".parse().unwrap());
                let mut res = fetch.send_with_signal(&signal).await?;
                let text = res.text().await?;
                Ok::<String, worker::Error>(text)
            };
            let delay_fut = async {
                Delay::from(Duration::from_millis(100)).await;
                controller.abort();
                Response::ok("Cancelled")
            };

            futures_util::pin_mut!(fetch_fut);
            futures_util::pin_mut!(delay_fut);

            match futures_util::future::select(delay_fut, fetch_fut).await {
                Either::Left((res, cancelled_fut)) => {
                    // Ensure that the cancelled future returns an AbortError.
                    match cancelled_fut.await {
                        Err(e) if e.to_string().starts_with("AbortError") => { /* Yay! It worked, let's do nothing to celebrate */},
                        Err(e) => panic!("Fetch errored with a different error than expected: {:#?}", e),
                        Ok(text) => panic!("Fetch unexpectedly succeeded: {}", text)
                    }

                    res
                },
                Either::Right(_) => panic!("Delay future should have resolved first"),
            }
        })
        .get("/redirect-default", |_, _| {
            Response::redirect("https://example.com".parse().unwrap())
        })
        .get("/redirect-307", |_, _| {
            Response::redirect_with_status("https://example.com".parse().unwrap(), 307)
        })
        .get("/now", |_, _| {
            let now = chrono::Utc::now();
            let js_date: Date = now.into();
            Response::ok(js_date.to_string())
        })
        .get_async("/cloned", |_, _| async {
            let mut resp = Response::ok("Hello")?;
            let mut resp1 = resp.cloned()?;

            let left = resp.text().await?;
            let right = resp1.text().await?;

            Response::ok((left == right).to_string())
        })
        .get_async("/cloned-stream", |_, _| async {
            let stream = futures_util::stream::repeat(())
                .take(10)
                .enumerate()
                .then(|(index, _)| async move {
                    Delay::from(Duration::from_millis(100)).await;
                    Result::Ok(index.to_string().into_bytes())
                });

            let mut resp = Response::from_stream(stream)?;
            let mut resp1 = resp.cloned()?;

            let left = resp.text().await?;
            let right = resp1.text().await?;

            Response::ok((left == right).to_string())
        })
        .get_async("/cloned-fetch", |_, _| async {
            let mut resp = Fetch::Url(
                "https://jsonplaceholder.typicode.com/todos/1"
                    .parse()
                    .unwrap(),
            )
            .send()
            .await?;
            let mut resp1 = resp.cloned()?;

            let left = resp.text().await?;
            let right = resp1.text().await?;

            Response::ok((left == right).to_string())
        })
        .get_async("/wait/:delay", |_, ctx| async move {
            let delay: Delay = match ctx.param("delay").unwrap().parse() {
                Ok(delay) => Duration::from_millis(delay).into(),
                Err(_) => return Response::error("invalid delay", 400),
            };

            // Wait for the delay to pass
            delay.await;

            Response::ok("Waited!\n")
        })
        .get("/custom-response-body", |_, _| {
            Response::from_body(ResponseBody::Body(vec![b'h', b'e', b'l', b'l', b'o']))
        })
        .get("/init-called", |_, _| {
            let init_called = GLOBAL_STATE.load(Ordering::SeqCst);
            Response::ok(init_called.to_string())
        })
        .get_async("/cache-example", |req, _| async move {
            console_log!("url: {}", req.url()?.to_string());
            let cache = Cache::default();
            let key = req.url()?.to_string();
            if let Some(resp) = cache.get(&key, true).await? {
                console_log!("Cache HIT!");
                Ok(resp)
            } else {

                console_log!("Cache MISS!");
                let mut resp = Response::from_json(&serde_json::json!({ "timestamp": Date::now().as_millis() }))?;

                // Cache API respects Cache-Control headers. Setting s-max-age to 10
                // will limit the response to be in cache for 10 seconds max
                resp.headers_mut().set("cache-control", "s-maxage=10")?;
                cache.put(key, resp.cloned()?).await?;
                Ok(resp)
            }
        })
        .get_async("/cache-api/get/:key", |_req, ctx| async move {
            if let Some(key) = ctx.param("key") {
                let cache = Cache::default();
                if let Some(resp) = cache.get(format!("https://{key}"), true).await? {
                    return Ok(resp);
                } else {
                    return Response::ok("cache miss");
                }
            }
            Response::error("key missing", 400)
        })
        .put_async("/cache-api/put/:key", |_req, ctx| async move {
            if let Some(key) = ctx.param("key") {
                let cache = Cache::default();

                let mut resp = Response::from_json(&serde_json::json!({ "timestamp": Date::now().as_millis() }))?;

                // Cache API respects Cache-Control headers. Setting s-max-age to 10
                // will limit the response to be in cache for 10 seconds max
                resp.headers_mut().set("cache-control", "s-maxage=10")?;
                cache.put(format!("https://{key}"), resp.cloned()?).await?;
                return Ok(resp);
            }
            Response::error("key missing", 400)
        })
        .post_async("/cache-api/delete/:key", |_req, ctx| async move {
            if let Some(key) = ctx.param("key") {
                let cache = Cache::default();

                let res = cache.delete(format!("https://{key}"), true).await?;
                return Response::ok(serde_json::to_string(&res)?);
            }
            Response::error("key missing", 400)
        })
        .get_async("/cache-stream", |req, _| async move {
            console_log!("url: {}", req.url()?.to_string());
            let cache = Cache::default();
            let key = req.url()?.to_string();
            if let Some(resp) = cache.get(&key, true).await? {
                console_log!("Cache HIT!");
                Ok(resp)
            } else {
                console_log!("Cache MISS!");
                let mut rng = rand::thread_rng();
                let count = rng.gen_range(0..10);
                let stream = futures_util::stream::repeat("Hello, world!\n")
                    .take(count)
                    .then(|text| async move {
                        Delay::from(Duration::from_millis(50)).await;
                        Result::Ok(text.as_bytes().to_vec())
                    });

                let mut resp = Response::from_stream(stream)?;
                console_log!("resp = {:?}", resp);
                // Cache API respects Cache-Control headers. Setting s-max-age to 10
                // will limit the response to be in cache for 10 seconds max
                resp.headers_mut().set("cache-control", "s-maxage=10")?;

                cache.put(key, resp.cloned()?).await?;
                Ok(resp)
            }
        })
        .get_async("/remote-by-request", |req, ctx| async move {
            let fetcher = ctx.service("remote")?;
            fetcher.fetch_request(req).await
        })
        .get_async("/remote-by-path", |req, ctx| async move {
            let fetcher = ctx.service("remote")?;
            let mut init = RequestInit::new();
            init.with_method(Method::Post);

            fetcher.fetch(req.url()?.to_string(), Some(init)).await
        })
        .post_async("/queue/send/:id", |_req, ctx| async move {
            let id = match ctx.param("id").map(|id|Uuid::try_parse(id).ok()).and_then(|u|u) {
                Some(id) => id,
                None =>  {
                    return Response::error("Failed to parse id, expected a UUID", 400);
                }
            };
            let my_queue = match ctx.env.queue("my_queue") {
                Ok(queue) => queue,
                Err(err) => {
                    return Response::error(format!("Failed to get queue: {err:?}"), 500)
                }
            };
            match my_queue.send(&QueueBody {
                id,
                id_string: id.to_string(),
            }).await {
                Ok(_) => {
                    Response::ok("Message sent")
                }
                Err(err) => {
                    Response::error(format!("Failed to send message to queue: {err:?}"), 500)
                }
            }
        }).get_async("/queue", |_req, _ctx| async move {
            let guard = GLOBAL_QUEUE_STATE.lock().unwrap();
            let messages: Vec<QueueBody> = guard.clone();
            Response::from_json(&messages)
        })
        .post_async("/d1/exec", |mut req, ctx| async move {
            let d1 = ctx.env.d1("DB")?;
            let query = req.text().await?;
            let exec_result = d1.exec(&query).await;
            match exec_result {
                Ok(result) => {
                    let count = result.count().unwrap_or(u32::MAX);
                    Response::ok(format!("{}", count))
                },
                Err(err) => Response::error(format!("Exec failed - {}", err), 500)
            }
            
        })
        .get_async("/r2/list-empty", r2::list_empty)
        .get_async("/r2/list", r2::list)
        .get_async("/r2/get-empty", r2::get_empty)
        .get_async("/r2/get", r2::get)
        .put_async("/r2/put", r2::put)
        .put_async("/r2/put-properties", r2::put_properties)
        .put_async("/r2/put-multipart", r2::put_multipart)
        .delete_async("/r2/delete", r2::delete)
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

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct QueueBody {
    pub id: Uuid,
    pub id_string: String,
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
