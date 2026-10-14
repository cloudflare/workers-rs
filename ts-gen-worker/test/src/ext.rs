//! Routes under `/ext/*` exercising the `#[event]` macros and the `impl` layer
//! (`parity/crate.md`). The suite's own `fetch` export is `respond_with_errors`, so the
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

#[derive(Serialize, Deserialize, Clone)]
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
        let count = storage
            .get::<JsValue>("count")
            .await?
            .as_f64()
            .unwrap_or(0.0)
            + 1.0;
        storage.put("count", &JsValue::from(count)).await?;
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
            })?;
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
            let keys = headers
                .keys()
                .into_iter()
                .map(|k| String::from(k.unwrap()))
                .collect::<Vec<_>>()
                .join(",");
            let values = headers
                .values()
                .into_iter()
                .map(|k| String::from(k.unwrap()))
                .collect::<Vec<_>>()
                .join(",");
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
                String::from(from_millis.to_iso_string()),
                now.as_millis() > from_millis.as_millis()
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
