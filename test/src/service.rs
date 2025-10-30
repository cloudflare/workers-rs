use super::SomeSharedData;
#[cfg(feature = "http")]
use std::convert::TryInto;
use worker::{Env, Method, Request, RequestInit, Response, Result};

pub async fn handle_remote_by_request(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let fetcher = env.service("remote")?;

    let response = fetcher.fetch_request(req).await?;

    #[cfg(feature = "http")]
    let result = Ok(TryInto::<worker::Response>::try_into(response)?);
    #[cfg(not(feature = "http"))]
    let result = Ok(response);

    result
}

pub async fn handle_remote_by_path(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let fetcher = env.service("remote")?;
    let mut init = RequestInit::new();
    init.with_method(Method::Post);
    let response = fetcher.fetch(req.url()?.to_string(), Some(init)).await?;

    #[cfg(feature = "http")]
    let result = Ok(TryInto::<worker::Response>::try_into(response)?);
    #[cfg(not(feature = "http"))]
    let result = Ok(response);

    result
}

// Compile-time assertion: public async Fetcher methods return Send futures.
#[allow(dead_code, unused)]
fn _assert_send() {
    fn require_send<T: Send>(_t: T) {}
    fn fetcher(f: worker::Fetcher) {
        require_send(f.fetch("https://example.com", None));
        require_send(f.fetch_request(
            worker::Request::new("https://example.com", worker::Method::Get).unwrap(),
        ));
    }
}
