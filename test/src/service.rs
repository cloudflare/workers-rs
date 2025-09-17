use super::SomeSharedData;
#[cfg(feature = "http")]
use std::convert::TryInto;
use worker::{Env, Method, Request, RequestInit, Response, Result};

#[worker::send]
pub async fn handle_remote_by_request(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let fetcher = env.service("remote")?;

    #[cfg(feature = "http")]
    let http_request = req.try_into()?;
    #[cfg(not(feature = "http"))]
    let http_request = req;

    let response = fetcher.fetch_request(http_request).await?;

    #[cfg(feature = "http")]
    let result = Ok(TryInto::<worker::Response>::try_into(response)?);
    #[cfg(not(feature = "http"))]
    let result = Ok(response);

    result
}

#[worker::send]
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
