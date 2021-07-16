use serde::{Deserialize, Serialize};
use worker::{kv::KvStore, prelude::*, Router};

mod utils;

#[derive(Deserialize, Serialize)]
struct MyData {
    message: String,
    #[serde(default)]
    is: bool,
    #[serde(default)]
    data: Vec<u8>,
}

#[derive(Deserialize)]
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

fn handle_a_request(_req: Request, _params: Params) -> Result<Response> {
    Response::ok("weeee".into())
}

#[cf::worker(fetch)]
pub async fn main(req: Request) -> Result<Response> {
    utils::set_panic_hook();

    let mut router = Router::new();

    router.get("/request", handle_a_request)?;
    router.post("/headers", |req, _| {
        let mut headers: http::HeaderMap = req.headers().into();
        headers.append("Hello", "World!".parse().unwrap());

        // TODO: make api for Response new and mut to add headers
        Response::ok("returned your headers to you.".into())
            .map(|res| res.with_headers(headers.into()))
    })?;

    router.on("/user/:id/test", |req, params| {
        if !matches!(req.method(), Method::Get) {
            return Response::error("Method Not Allowed".into(), 405);
        }
        let id = params.get("id").unwrap_or("not found");
        Response::ok(format!("TEST user id: {}", id))
    })?;

    router.on("/user/:id", |_req, params| {
        let id = params.get("id").unwrap_or("not found");
        Response::from_json(&User {
            id: id.into(),
            timestamp: Date::now().as_millis(),
            date_from_int: Date::new(DateInit::Millis(1234567890)).to_string(),
            date_from_str: Date::new(DateInit::String(
                "Wed Jan 14 1980 23:56:07 GMT-0700 (Mountain Standard Time)".into(),
            ))
            .to_string(),
        })
    })?;

    router.post("/account/:id/zones", |_, params| {
        Response::ok(format!(
            "Create new zone for Account: {}",
            params.get("id").unwrap_or("not found")
        ))
    })?;

    router.get("/account/:id/zones", |_, params| {
        Response::ok(format!(
            "Account id: {}..... You get a zone, you get a zone!",
            params.get("id").unwrap_or("not found")
        ))
    })?;

    // Router currently only supports synchronous functions as callbacks
    // So the async ones are handled separately
    match (req.method(), req.path().as_str()) {
        (_, "/fetch") => {
            let req = Request::new("https://example.com", "POST")?;
            let resp = Fetch::Request(&req).fetch().await?;
            let resp2 = Fetch::Url("https://example.com").fetch().await?;
            Response::ok(format!(
                "received responses with codes {} and {}",
                resp.status_code(),
                resp2.status_code()
            ))
        }
        (_, "/proxy_request") => Fetch::Url("https://example.com").fetch().await,
        (_, "/fetch_json") => {
            let data: ApiData = Fetch::Url("https://jsonplaceholder.typicode.com/todos/1")
                .fetch()
                .await?
                .json()
                .await?;
            Response::ok(format!(
                "API Returned user: {} with title: {} and completed: {}",
                data.user_id, data.title, data.completed
            ))
        }
        _ => router.run(req),
    }

    // match (req.method(), req.path().as_str()) {
    //     (Method::Get, "/") => {
    //         let msg = format!(
    //             "[rustwasm] event type: {}, colo: {}, asn: {}",
    //             req.event_type(),
    //             req.cf().colo(),
    //             req.cf().asn(),
    //         );
    //         Response::ok(Some(msg))
    //     }
    //     (Method::Post, "/") => {
    //         let data: MyData = req.json().await?;
    //         Response::ok(Some(format!("[POST /] message = {}", data.message)))
    //     }
    //     (Method::Post, "/read-text") => Response::ok(Some(format!(
    //         "[POST /read-text] text = {}",
    //         req.text().await?
    //     ))),
    //     (_, "/json") => Response::json(&MyData {
    //         message: "hello!".into(),
    //         is: true,
    //         data: vec![1, 2, 3, 4, 5],
    //     }),
    // (Method::Get, "/headers") => {
    //     for (_, value) in req.headers() {
    //         if &value == "evil value" {
    //             return Response::error("stop that!".into(), 400);
    //         }
    //     }
    //     let msg = req
    //         .headers()
    //         .into_iter()
    //         .map(|(name, value)| format!("{}: {}\n", name, value))
    //         .collect();
    //     let mut headers: worker::Headers = [
    //         ("Content-Type", "application/json"),
    //         ("Set-Cookie", "hello=true"),
    //     ]
    //     .iter()
    //     .collect();
    //     headers.append("Set-Cookie", "world=true")?;
    //     Response::ok(Some(msg)).map(|res| res.with_headers(headers))
    // }
    // (Method::Post, "/headers") => {
    //     let mut headers: http::HeaderMap = req.headers().into();
    //     headers.append("Hello", "World!".parse().unwrap());
    //     Response::ok(Some("returned your headers to you.".into()))
    //         .map(|res| res.with_headers(headers.into()))
    // }
    //     (Method::Post, "/job") => {
    //         let kv = KvStore::create("JOB_LOG").expect("no binding for JOB_LOG");
    //         if kv
    //             .put("manual entry", 123)
    //             .expect("fail to build KV put operation")
    //             .execute()
    //             .await
    //             .is_err()
    //         {
    //             return Response::error("Failed to put into KV".into(), 500);
    //         } else {
    //             return Response::empty();
    //         }
    //     }
    //     (_, "/jobs") => {
    //         if let Ok(kv) = KvStore::create("JOB_LOG") {
    //             return match kv.list().execute().await {
    //                 Ok(jobs) => Response::json(&jobs),
    //                 Err(e) => Response::error(format!("KV list error: {:?}", e), 500),
    //             };
    //         }
    //         Response::error("Failed to access KV binding".into(), 500)
    //     }
    //     (_, "/404") => Response::error("Not Found".to_string(), 404),
    //     _ => Response::ok(Some(format!("{:?} {}", req.method(), req.path()))),
    // }
}

#[cf::worker(scheduled)]
pub async fn job(s: Schedule) -> Result<()> {
    utils::set_panic_hook();

    let kv = KvStore::create("JOB_LOG").expect("no binding for JOB_LOG");
    kv.put(&format!("{}", s.time()), s)
        .expect("fail to build KV put operation")
        .execute()
        .await
        .map_err(worker::Error::from)

    // s.time() = 1621579157181, s.cron() = "15 * * * *", s.event_type() == "scheduled";
}
