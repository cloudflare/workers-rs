//! Routes under `/ext/*` exercising the `#[event]` macros and the `impl` layer.
//! The suite's own `fetch` export is `respond_with_errors`, so the
//! default error policy is only exercised through `__macro::error_response`.

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use ts_gen_worker::*;
use wasm_bindgen::prelude::*;

use crate::Env;

static EXT_STARTED: AtomicBool = AtomicBool::new(false);

thread_local! {
    static SCHEDULED: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static EMAILED: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static SPAWNED: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Body {
    id: String,
    count: u32,
}

#[derive(Deserialize)]
struct Query {
    name: String,
    n: u32,
}

#[event(start)]
fn ext_start() {
    EXT_STARTED.store(true, Ordering::SeqCst);
}

#[event(scheduled)]
async fn scheduled(controller: ScheduledController, _env: Env, _ctx: ExecutionContext) {
    SCHEDULED.with_borrow_mut(|v| {
        v.push(format!(
            "{} {}",
            controller.cron(),
            controller.scheduled_time()
        ))
    });
}

#[event(email)]
async fn email(message: ForwardableEmailMessage, _env: Env, _ctx: ExecutionContext) -> Result<()> {
    let subject = message.headers().get("subject")?.unwrap_or_default();
    EMAILED.with_borrow_mut(|v| {
        v.push(format!(
            "{} -> {} ({}): {subject}",
            message.from(),
            message.to(),
            message.raw_size()
        ))
    });
    if subject == "reject" {
        message.set_reject("not wanted")?;
    }
    Ok(())
}

#[durable_object]
struct MacroCounter {
    state: DurableObjectState,
    env: Env,
}

impl DurableObject for MacroCounter {
    type Env = Env;

    fn new(state: DurableObjectState, env: Env) -> Self {
        Self { state, env }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let storage = self.state.storage();
        if req.path() == "/serde" {
            let before: Option<Body> = storage.get_into("body").await?;
            let body = Body {
                id: "s".into(),
                count: before.map_or(0, |b| b.count) + 1,
            };
            storage.put_serialized("body", &body).await?;
            storage.put_serialized("n", &7u32).await?;
            let n: Option<u32> = storage.get_into("n").await?;
            let missing: Option<String> = storage.get_into("missing").await?;
            let after: Option<Body> = storage.get_into("body").await?;
            return Response::builder().ok(format!("{after:?} {n:?} {missing:?}"));
        }
        let count = storage.get::<f64>("count").await?.unwrap_or(0.0) + 1.0;
        storage.put("count", &count).await?;
        let name = self.state.id().name().unwrap_or_default();
        Response::builder().ok(format!(
            "{name} {count} {} {}",
            req.path(),
            self.env.greeting()
        ))
    }
}

fn router<'a>() -> Router<'a, Env, u32> {
    Router::<Env, u32>::with_data(7)
        .get("/ext/router/sync/:id", |_req, ctx| {
            Response::builder().ok(format!("sync {} {}", ctx.param("id").unwrap(), ctx.data))
        })
        .post_async("/ext/router/async/:id", |req, ctx| async move {
            let body = req.text().await?;
            Response::builder().ok(format!(
                "async {} {body} {}",
                ctx.param("id").unwrap(),
                ctx.env.greeting()
            ))
        })
        .or_else_any_method("/ext/router/any", |req, _ctx| {
            Response::builder().ok(format!("any {}", req.method()))
        })
}

fn assert_send<T: Send>(_: &T) {}

#[send]
async fn send_handler(env: Env) -> Result<String> {
    // A JS handle held across an await would make this future !Send without the macro.
    let stub = env.macro_counter().get_by_name("send")?;
    let res = stub.fetch_with_str("https://do/send").await?;
    res.text().await
}

pub async fn handle(
    req: Request,
    env: Env,
    ctx: ExecutionContext,
    segments: &[&str],
) -> Result<Response> {
    match (req.method().as_str(), segments) {
        ("GET", ["start"]) => {
            Response::builder().ok(format!("started={}", EXT_STARTED.load(Ordering::SeqCst)))
        }

        ("GET", ["spawn"]) => {
            ctx.spawn(async {
                let _ = fetch_with_str("https://upstream.test/data").await;
                SPAWNED.with_borrow_mut(|v| v.push("done".into()));
            });
            Response::builder().ok("spawned")
        }
        ("GET", ["spawned"]) => Response::builder().ok(SPAWNED.with_borrow(|v| v.join(","))),

        ("GET", ["scheduled"]) => Response::builder().ok(SCHEDULED.with_borrow(|v| v.join(","))),
        ("GET", ["email"]) => Response::builder().ok(EMAILED.with_borrow(|v| v.join(","))),

        ("GET", ["method"]) => {
            let method: Method = req.method().parse()?;
            let all = Method::all()
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            let bad = "PURGE"
                .parse::<Method>()
                .err()
                .map(|e| e.to_string())
                .unwrap_or_default();
            Response::builder().ok(format!("{method} {:?} | {all} | {bad}", Method::Get))
        }

        ("GET", ["path"]) => {
            let query: Query = req.query()?;
            Response::builder().ok(format!("{} {} {}", req.path(), query.name, query.n))
        }

        ("GET", ["headers-iter"]) => {
            let headers: Headers = [("x-a", "1"), ("x-b", "2")].into_iter().collect();
            headers.append("x-a", "3")?;
            headers.append("set-cookie", "a=1")?;
            headers.append("set-cookie", "b=2")?;
            let entries = (&headers)
                .into_iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join(",");
            let keys = headers.keys().into_iter().collect::<Vec<_>>().join(",");
            let values = headers.values().into_iter().collect::<Vec<_>>().join(",");
            let all = headers.get_all("set-cookie")?.join("|");
            let x_a = headers.get_all("x-a").unwrap_err().to_string();
            Response::builder().ok(format!("{entries} / {keys} / {values} / {all} / {x_a}"))
        }

        ("POST", ["json-into"]) => {
            let mut body: Body = req.json_into().await?;
            body.count += 1;
            Response::builder().status(201).from_json(&body)
        }

        ("POST", ["bytes-stream"]) => {
            let mut stream = req.bytes_stream().ok_or_else(|| Error::from("no body"))?;
            let mut chunks = Vec::new();
            while let Some(chunk) = stream.next().await {
                chunks.push(chunk?);
            }
            let total: usize = chunks.iter().map(Vec::len).sum();
            let out = futures_util::stream::iter(chunks.into_iter().map(Ok::<_, Error>));
            let headers: Headers = [("x-total", total.to_string())].into_iter().collect();
            Response::builder().headers(&headers).from_stream(out)
        }

        ("GET", ["bytes"]) => {
            let bytes = req.bytes().await?.to_vec();
            Response::builder().from_bytes(bytes)
        }

        ("GET", ["html"]) => Response::builder().status(201).from_html("<b>hi</b>"),
        ("GET", ["empty"]) => Response::builder().status(204).empty(),
        ("GET", ["teapot"]) => Response::builder().error("short and stout", 418),
        ("GET", ["bad-error"]) => Response::builder().error("nope", 200),
        ("GET", ["ok-ct"]) => {
            let headers: Headers = [("content-type", "text/csv")].into_iter().collect();
            Response::builder().headers(&headers).ok("a,b")
        }

        (_, ["router", ..]) => router().run(req, env).await,

        ("GET", ["counter"]) => {
            let stub = env.macro_counter().get_by_name("main")?;
            let res = stub.fetch_with_str("https://do/hit").await?;
            Response::builder().ok(res.text().await?)
        }

        ("GET", ["console"]) => {
            console_log!("ext log {} {}", 1, "a");
            console_debug!("ext debug {}", 2);
            console_warn!("ext warn {}", 3);
            console_error!("ext error {}", 4);
            Response::builder().ok("logged")
        }

        ("GET", ["delay"]) => {
            let start = js_sys::Date::now();
            // The losing 30 s delay is dropped while pending, which clears its timer.
            let first = futures_util::future::select(
                Delay::from(Duration::from_secs(30)),
                Delay::from(Duration::from_millis(100)),
            )
            .await;
            let elapsed = js_sys::Date::now() - start;
            let short_won = matches!(first, futures_util::future::Either::Right(_));
            Response::builder().ok(format!("{short_won} {}", elapsed >= 100.0))
        }

        ("GET", ["date"]) => {
            let from_millis: js_sys::Date = DateInit::Millis(1_630_611_511_000).into();
            let from_str = js_sys::Date::from_init(DateInit::String(
                "Thu, 02 Sep 2021 19:38:31 GMT".to_owned(),
            ));
            let now = js_sys::Date::current();
            Response::builder().ok(format!(
                "{} {} {} {}",
                from_millis.as_millis(),
                from_str.as_millis(),
                from_millis.to_iso_string(),
                now.as_millis() > from_millis.as_millis()
            ))
        }

        ("GET", ["cors"]) => {
            let cors = Cors::new()
                .with_credentials(true)
                .with_max_age(600)
                .with_origins(["https://a.test", "https://b.test"])
                .with_methods([Method::Get, Method::Post])
                .with_allowed_headers(["x-a", "x-b"])
                .with_exposed_headers(["x-c"]);
            let headers = Headers::default();
            Cors::new().apply_headers(&headers)?;
            let untouched = headers.entries().into_iter().count();
            Response::builder()
                .ok(format!("untouched={untouched}"))?
                .with_cors(&cors)
        }

        ("GET", ["env-ext"]) => {
            let var = env.var("GREETING")?;
            let secret = env.secret("GREETING")?;
            let ns: DurableObjectNamespace = env.get("MACRO_COUNTER")?;
            let name = ns.id_from_name("x")?.name().unwrap_or_default();
            let missing = env.get::<KVNamespace>("NOPE").unwrap_err().to_string();
            let not_string = env.var("MACRO_COUNTER").unwrap_err().to_string();
            let json: Body = env.object_var("EXT_JSON")?;
            Response::builder().ok(format!(
                "{var} | {secret} | {name} | {missing} | {not_string} | {json:?}"
            ))
        }

        ("GET", ["send"]) => {
            let fut = send_handler(env.clone());
            assert_send(&fut);
            let body = fut.await?;
            let wrapped = send::SendWrapper::new(JsValue::from("w"));
            assert_send(&wrapped);
            let via_trait = send::IntoSendFuture::into_send(async { 1 });
            assert_send(&via_trait);
            Response::builder().ok(format!(
                "{body} | {} | {}",
                wrapped.as_string().unwrap(),
                via_trait.await
            ))
        }

        ("GET", ["fetch-enum"]) => {
            let by_url = Fetch::Url("https://upstream.test/a".into()).send().await?;
            let req = Request::new_with_str_and_init(
                "https://upstream.test/b",
                &RequestInit::builder().method("POST").build(),
            )?;
            let by_request = Fetch::Request(req).send().await?;
            let controller = AbortController::default();
            controller.abort()?;
            let aborted = Fetch::from("https://upstream.test/c")
                .send_with_signal(&controller.signal())
                .await
                .unwrap_err()
                .to_string();
            Response::builder().ok(format!(
                "{} {} {} {} | {aborted}",
                by_url.status(),
                by_url.text().await?,
                by_request.status(),
                by_request.text().await?,
            ))
        }

        ("GET", ["kv"]) => {
            #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
            struct Meta {
                tag: String,
            }
            let kv = env.ext_kv();
            let body = Body {
                id: "k".into(),
                count: 3,
            };
            kv.put_builder("t", "text")?
                .expiration_ttl(60)
                .metadata(Meta { tag: "m".into() })?
                .execute()
                .await?;
            kv.put_builder("n", 42)?.execute().await?;
            kv.put_builder("j", &body)?.execute().await?;
            kv.put_bytes("b", &[1, 2, 3]).execute().await?;

            let text = kv.get_builder("t").text().await?;
            let (text2, meta): (Option<String>, Option<Meta>) =
                kv.get_builder("t").text_with_metadata().await?;
            let (_, no_meta): (Option<String>, Option<Meta>) =
                kv.get_builder("n").text_with_metadata().await?;
            let n: Option<u32> = kv.get_builder("n").cache_ttl(60).json().await?;
            let j: Option<Body> = kv.get_builder("j").json().await?;
            let bytes = kv.get_builder("b").bytes().await?;
            let missing = kv.get_builder("missing").text().await?;
            let stream = kv.get_builder("t").stream().await?.is_some();
            let no_stream = kv.get_builder("missing").stream().await?.is_some();

            let mut bulk: Vec<_> = kv
                .get_bulk(&["t", "n", "missing"])
                .text()
                .await?
                .into_iter()
                .collect();
            bulk.sort();
            let mut bulk_meta: Vec<_> = kv
                .get_bulk(&["t", "n"])
                .text_with_metadata::<Meta>()
                .await?
                .into_iter()
                .collect();
            bulk_meta.sort();

            let list = kv.list_builder().prefix("t").limit(10).execute().await?;
            let names: Vec<String> = list.keys().iter().map(|k| k.name()).collect();
            let all = kv.list_builder().execute().await?.keys().len();
            kv.delete("n").await?;
            let after = kv.list_builder().execute().await?.keys().len();

            Response::builder().ok(format!(
                "{text:?} {text2:?} {meta:?} {no_meta:?} {n:?} {j:?} {bytes:?} {missing:?} {stream} {no_stream} | {bulk:?} | {bulk_meta:?} | {names:?} {} {all} {after}",
                list.list_complete(),
            ))
        }

        ("GET", ["ok"]) => {
            let value = ok::Ok(Response::builder().status(202).ok("ok")?)?;
            match Ok::<u8, Error>(1) {
                Ok(1) => ok::Ok(value),
                _ => Err(Error::from("pattern shadowed")),
            }
        }

        ("GET", ["into-parts"]) => {
            let upstream = Response::builder()
                .status(200)
                .header("x-upstream", "yes")?
                .cf(&js_sys::JSON::parse(r#"{"foo":"bar"}"#)?)
                .ok("upstream body")?;
            let (builder, body) = upstream.into_parts();
            let res = builder.status(404).header("x-rewritten", "1")?.body(body)?;
            let (builder, body) = Response::builder().status(204).empty()?.into_parts();
            let empty = builder.body(body)?;
            let cf = js_sys::JSON::stringify(&res.cf())?.unwrap_or_default();
            Response::builder().ok(format!(
                "{} {} {} {} {cf} | {} {}",
                res.status(),
                res.headers().get("x-upstream")?.unwrap_or_default(),
                res.headers().get("x-rewritten")?.unwrap_or_default(),
                res.text().await?,
                empty.status(),
                empty.body().is_none(),
            ))
        }

        ("GET", ["builder-header"]) => {
            let cors = Cors::new().with_origins(["https://a.test"]);
            let res = Response::builder()
                .header("x-a", "1")?
                .header("x-a", "2")?
                .header("content-type", "text/csv")?
                .cors(&cors)?
                .encode_body(EncodeBodyKind::Manual)
                .ok("a,b")?;
            Response::builder().ok(format!(
                "{} {} {}",
                res.headers().get("x-a")?.unwrap_or_default(),
                res.headers().get("content-type")?.unwrap_or_default(),
                res.headers()
                    .get("access-control-allow-origin")?
                    .unwrap_or_default(),
            ))
        }

        ("GET", ["request-method"]) => {
            let req = Request::new_with_method("https://upstream.test/x", Method::Put)?;
            Response::builder().ok(format!("{} {}", req.method(), req.url()))
        }

        ("GET", ["from-websocket"]) => {
            let (client, server) = WebSocketPair::new()?.into();
            server.accept()?;
            let res = Response::builder().from_websocket(client)?;
            Response::builder().ok(format!("{} {}", res.status(), res.web_socket().is_some()))
        }

        ("GET", ["storage-serde"]) => {
            let stub = env.macro_counter().get_by_name("serde")?;
            Response::builder().ok(stub
                .fetch_with_str("https://do/serde")
                .await?
                .text()
                .await?)
        }

        ("POST", ["http-roundtrip"]) => {
            let req_body = req.body().ok_or_else(|| Error::from("no body"))?;
            let http_req: HttpRequest = req.try_into()?;
            let back: Request = http_req.try_into()?;
            let req_same = back.body().as_ref() == Some(&req_body);
            let text = back.text().await?;

            let out = futures_util::stream::iter([Ok::<_, Error>(b"stream".to_vec())]);
            let res = Response::builder().from_stream(out)?;
            let res_body = res.body().ok_or_else(|| Error::from("no body"))?;
            let http_res: HttpResponse = res.try_into()?;
            let back: Response = http_res.try_into()?;
            let res_same = back.body().as_ref() == Some(&res_body);
            let res_text = back.text().await?;

            // After a partial read, into_inner still returns the same object (now disturbed,
            // so the platform refuses it as a body, but a streams consumer can continue it).
            let partial = Response::builder().from_stream(futures_util::stream::iter([
                Ok::<_, Error>(b"one".to_vec()),
                Ok(b"two".to_vec()),
            ]))?;
            let partial_body = partial.body().ok_or_else(|| Error::from("no body"))?;
            let mut stream = ByteStream::from(partial_body.clone());
            let first =
                String::from_utf8_lossy(&stream.next().await.transpose()?.unwrap_or_default())
                    .into_owned();
            let partial_same = stream.into_inner().as_ref() == Some(&partial_body);
            let mut rest = String::new();
            let mut stream = ByteStream::from(partial_body);
            while let Some(chunk) = stream.next().await {
                rest.push_str(&String::from_utf8_lossy(&chunk?));
            }

            Response::builder().ok(format!(
                "{req_same} {text} | {res_same} {res_text} | {first} {partial_same} {rest}"
            ))
        }

        ("GET", ["header-map"]) => {
            let mut map = http::HeaderMap::new();
            map.append("x-a", http::HeaderValue::from_static("1"));
            map.append("x-a", http::HeaderValue::from_static("2"));
            let headers = Headers::try_from(&map)?;
            let back = http::HeaderMap::try_from(&headers)?;
            map.append(
                "x-bin",
                http::HeaderValue::from_bytes("h\u{e9}llo".as_bytes()).unwrap(),
            );
            let to_js = Headers::try_from(&map).unwrap_err().to_string();
            let del = Headers::default();
            del.set("x-del", "a\u{7f}b")?;
            let to_rust = http::HeaderMap::try_from(&del).unwrap_err().to_string();
            Response::builder().ok(format!(
                "{} {} | {to_js} | {to_rust}",
                headers.get("x-a")?.unwrap_or_default(),
                back.get_all("x-a").iter().count(),
            ))
        }

        ("GET", ["error-number"]) => Err(JsValue::from(42).into()),
        ("GET", ["error-null"]) => Err(JsValue::NULL.into()),
        ("GET", ["error-rust"]) => Err(Error::from("boom")),
        ("GET", ["error-default"]) => Ok(__macro::error_response(
            js_sys::Error::new("hidden").into(),
            None,
        )),

        _ => Response::builder().error("Not Found", 404),
    }
}
