use super::{ApiData, SomeSharedData};
use futures_util::future::Either;
use std::time::Duration;
use worker::{
    wasm_bindgen_futures, AbortController, Delay, Fetch, Method, Request, RequestInit, Response,
    Result, RouteContext,
};

pub async fn handle_fetch(_req: Request, _ctx: RouteContext<SomeSharedData>) -> Result<Response> {
    let req = Request::new("https://example.com", Method::Post)?;
    worker::console_log!("foo");
    let resp = Fetch::Request(req).send().await?;
    worker::console_log!("bar");
    let resp2 = Fetch::Url("https://example.com".parse()?).send().await?;
    worker::console_log!("baz");
    Response::ok(format!(
        "received responses with codes {} and {}",
        resp.status_code(),
        resp2.status_code()
    ))
}

pub async fn handle_fetch_json(
    _req: Request,
    _ctx: RouteContext<SomeSharedData>,
) -> Result<Response> {
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
}

pub async fn handle_proxy_request(
    _req: Request,
    ctx: RouteContext<SomeSharedData>,
) -> Result<Response> {
    let url = ctx.param("url").unwrap();
    Fetch::Url(url.parse()?).send().await
}

pub async fn handle_request_init_fetch(
    _req: Request,
    _ctx: RouteContext<SomeSharedData>,
) -> Result<Response> {
    let init = RequestInit::new();
    Fetch::Request(Request::new_with_init("https://cloudflare.com", &init)?)
        .send()
        .await
}

pub async fn handle_request_init_fetch_post(
    _req: Request,
    _ctx: RouteContext<SomeSharedData>,
) -> Result<Response> {
    let mut init = RequestInit::new();
    init.method = Method::Post;
    Fetch::Request(Request::new_with_init("https://httpbin.org/post", &init)?)
        .send()
        .await
}

pub async fn handle_cancelled_fetch(
    _req: Request,
    _ctx: RouteContext<SomeSharedData>,
) -> Result<Response> {
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
}

pub async fn handle_fetch_timeout(
    _req: Request,
    _ctx: RouteContext<SomeSharedData>,
) -> Result<Response> {
    let controller = AbortController::default();
    let signal = controller.signal();

    let fetch_fut = async {
        let fetch = Fetch::Url("https://miniflare.mocks/delay".parse().unwrap());
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
                Err(e) if e.to_string().contains("AbortError") => { /* Yay! It worked, let's do nothing to celebrate */
                }
                Err(e) => panic!(
                    "Fetch errored with a different error than expected: {:#?}",
                    e
                ),
                Ok(text) => panic!("Fetch unexpectedly succeeded: {}", text),
            }

            res
        }
        Either::Right(_) => panic!("Delay future should have resolved first"),
    }
}

pub async fn handle_cloned_fetch(
    _req: Request,
    _ctx: RouteContext<SomeSharedData>,
) -> Result<Response> {
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
}
