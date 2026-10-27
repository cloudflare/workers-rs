use super::{ApiData, SomeSharedData};
use crate::util::sleep;
use crate::Env;
use futures_util::future::Either;
use serde::{Deserialize, Serialize};
use ts_gen_worker::*;
use wasm_bindgen_futures::spawn_local;

pub async fn handle_fetch(_req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let req = Request::new_with_method("https://example.com", Method::Post)?;
    let resp = fetch(&req).await?;
    let resp2 = fetch_with_str("https://example.com").await?;
    Response::builder().ok(format!(
        "received responses with codes {} and {}",
        resp.status(),
        resp2.status()
    ))
}

pub async fn handle_fetch_json(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let data: ApiData = fetch_with_str("https://jsonplaceholder.typicode.com/todos/1")
        .await?
        .json_into()
        .await?;
    Response::builder().ok(format!(
        "API Returned user: {} with title: {} and completed: {}",
        data.user_id, data.title, data.completed
    ))
}

pub async fn handle_proxy_request(
    req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let uri = req.url_parsed()?;
    let url = uri
        .path_segments()
        .unwrap()
        .skip(1)
        .collect::<Vec<_>>()
        .join("/");
    console_log!("{url}");
    fetch_with_str(&url).await
}

pub async fn handle_request_init_fetch(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let init = RequestInit::new();
    fetch(&Request::new_with_str_and_init(
        "https://cloudflare.com",
        &init,
    )?)
    .await
}

pub async fn handle_request_init_fetch_post(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let init = RequestInit::builder().method("POST").build();
    fetch(&Request::new_with_str_and_init(
        "https://httpbin.org/post",
        &init,
    )?)
    .await
}

pub async fn handle_cancelled_fetch(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let controller = AbortController::new();
    let signal = controller.signal();

    let (tx, rx) = futures_channel::oneshot::channel();

    spawn_local({
        async move {
            let init = RequestInit::builder().signal(&signal).build();
            let res = fetch_with_str_and_init("https://cloudflare.com", &init).await;
            tx.send(res).unwrap();
        }
    });

    controller.abort()?;

    let res = rx.await.unwrap();
    let res = res.unwrap_or_else(|e| Response::builder().ok(e.to_string()).unwrap());

    Ok(res)
}

pub async fn handle_fetch_timeout(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let controller = AbortController::new();
    let signal = controller.signal();

    let fetch_fut = async {
        let init = RequestInit::builder().signal(&signal).build();
        let res = fetch_with_str_and_init("https://miniflare.mocks/delay", &init).await?;
        let text = res.text().await?;
        Ok::<String, Error>(text)
    };
    let delay_fut = async {
        sleep(100.0).await;
        controller.abort()?;
        Response::builder().ok("Cancelled")
    };

    futures_util::pin_mut!(fetch_fut);
    futures_util::pin_mut!(delay_fut);

    match futures_util::future::select(delay_fut, fetch_fut).await {
        Either::Left((res, cancelled_fut)) => {
            match cancelled_fut.await {
                Err(e) if e.to_string().contains("AbortError") => {}
                Err(e) => panic!(
                    "Fetch errored with a different error than expected: {:#?}",
                    e.to_string()
                ),
                Ok(text) => panic!("Fetch unexpectedly succeeded: {text}"),
            }

            res
        }
        Either::Right(_) => panic!("Delay future should have resolved first"),
    }
}

pub async fn handle_cloned_fetch(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let resp = fetch_with_str("https://jsonplaceholder.typicode.com/todos/1").await?;
    let resp1 = resp.clone()?;

    let left = resp.text().await?;
    let right = resp1.text().await?;

    Response::builder().ok((left == right).to_string())
}

pub async fn handle_cloned_response_attributes(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestCf {
        foo: String,
    }
    let cf = serde_wasm_bindgen::to_value(&TestCf {
        foo: "bar".to_owned(),
    })?;
    let resp = Response::new_with_str_and_init(
        "",
        &ResponseInit::builder()
            .status(200)
            .encode_body(EncodeBodyKind::Manual)
            .cf(&cf)
            .build(),
    )?;

    let resp1 = resp.clone()?;

    let cf: TestCf = serde_wasm_bindgen::from_value(resp.cf())?;
    let cf1: TestCf = serde_wasm_bindgen::from_value(resp1.cf())?;

    assert_eq!(cf, cf1);

    Response::builder().ok("true")
}
