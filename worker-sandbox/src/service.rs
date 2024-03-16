use super::SomeSharedData;
#[cfg(feature = "http")]
use std::convert::TryInto;
use worker::{Method, Request, RequestInit, Response, Result, RouteContext};

pub async fn handle_remote_by_request(
    req: Request,
    ctx: RouteContext<SomeSharedData>,
) -> Result<Response> {
    let fetcher = ctx.service("remote")?;

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

pub async fn handle_remote_by_path(
    req: Request,
    ctx: RouteContext<SomeSharedData>,
) -> Result<Response> {
    let fetcher = ctx.service("remote")?;
    let mut init = RequestInit::new();
    init.with_method(Method::Post);
    let response = fetcher.fetch(req.url()?.to_string(), Some(init)).await?;

    #[cfg(feature = "http")]
    let result = Ok(TryInto::<worker::Response>::try_into(response)?);
    #[cfg(not(feature = "http"))]
    let result = Ok(response);

    result
}
