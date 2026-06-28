use std::time::Duration;

use futures_util::future::Either;
use worker::{
    event, AbortController, AbortSignal, Context, Delay, Env, Fetch, Request, Response, Result,
    RouteContext, Router,
};

fn get_target_url(req: &Request) -> Result<String> {
    req.url()?
        .query_pairs()
        .find(|(k, _)| k == "url")
        .map(|(_, v)| v.into_owned())
        .ok_or_else(|| worker::Error::RustError("Missing 'url' query param".into()))
}

async fn abort_immediate(req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let target = get_target_url(&req)?;

    let signal = AbortSignal::abort();
    let fetch = Fetch::Url(target.parse()?);

    match fetch.send_with_signal(&signal).await {
        Ok(mut resp) => {
            let text = resp.text().await?;
            Response::ok(format!("Unexpected success: {text}"))
        }
        Err(e) => Response::ok(format!("Aborted: {e}")),
    }
}

async fn abort_timeout(req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let target = get_target_url(&req)?;

    let timeout_ms: u64 = req
        .url()?
        .query_pairs()
        .find(|(k, _)| k == "timeout")
        .and_then(|(_, v)| v.parse().ok())
        .unwrap_or(2000);

    let url = target.parse()?;
    let controller = AbortController::default();
    let signal = controller.signal();

    let fetch_fut = async {
        let mut resp = Fetch::Url(url).send_with_signal(&signal).await?;
        let text = resp.text().await?;
        Ok::<_, worker::Error>(text)
    };

    let timeout_fut = async {
        Delay::from(Duration::from_millis(timeout_ms)).await;
        controller.abort();
    };

    futures_util::pin_mut!(fetch_fut);
    futures_util::pin_mut!(timeout_fut);

    match futures_util::future::select(timeout_fut, fetch_fut).await {
        Either::Left((_timed_out, _cancelled)) => {
            Response::ok(format!("Request timed out after {timeout_ms}ms"))
        }
        Either::Right((Ok(body), _)) => Response::ok(format!("Got response: {body}")),
        Either::Right((Err(e), _)) => Response::ok(format!("Fetch error: {e}")),
    }
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get_async("/abort", abort_immediate)
        .get_async("/timeout", abort_timeout)
        .run(req, env)
        .await
}
