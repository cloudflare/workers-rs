use serde::{Deserialize, Serialize};
use worker::{kv::KvStore, Method, Request, Response, Result, Schedule};

mod utils;

#[derive(Deserialize, Serialize)]
struct MyData {
    message: String,
    #[serde(default)]
    is: bool,
    #[serde(default)]
    data: Vec<u8>,
}

#[cf::worker(fetch)]
pub async fn main(mut req: Request) -> Result<Response> {
    utils::set_panic_hook();

    match (req.method(), req.path().as_str()) {
        (Method::Get, "/") => {
            let msg = format!(
                "[rustwasm] event type: {}, colo: {}, asn: {}",
                req.event_type(),
                req.cf().colo(),
                req.cf().asn(),
            );
            Response::ok(Some(msg))
        }
        (Method::Post, "/") => {
            let data: MyData = req.json().await?;
            Response::ok(Some(format!("[POST /] message = {}", data.message)))
        }
        (Method::Post, "/read-text") => Response::ok(Some(format!(
            "[POST /read-text] text = {}",
            req.text().await?
        ))),
        (_, "/json") => Response::json(&MyData {
            message: "hello!".into(),
            is: true,
            data: vec![1, 2, 3, 4, 5],
        }),
        (Method::Post, "/job") => {
            let kv = KvStore::create("JOB_LOG").expect("no binding for JOB_LOG");
            if kv
                .put("manual entry", 123)
                .expect("fail to build KV put operation")
                .execute()
                .await
                .is_err()
            {
                return Response::error("Failed to put into KV".into(), 500);
            } else {
                return Response::empty();
            }
        }
        (_, "/jobs") => {
            if let Ok(kv) = KvStore::create("JOB_LOG") {
                return match kv.list().execute().await {
                    Ok(jobs) => Response::json(&jobs),
                    Err(e) => Response::error(format!("KV list error: {:?}", e), 500),
                };
            }
            Response::error("Failed to access KV binding".into(), 500)
        }
        (_, "/404") => Response::error("Not Found".to_string(), 404),
        _ => Response::ok(Some(format!("{:?} {}", req.method(), req.path()))),
    }
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
