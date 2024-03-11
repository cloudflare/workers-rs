use wasm_bindgen::JsCast;
use web_sys::WorkerGlobalScope;

use crate::{
    body::Body,
    futures::SendJsFuture,
    http::{request, response},
    AbortSignal, Error, RequestInit, Result,
};

/// Fetch a resource from the network.
///
/// # Example
///
/// ```rust,no_run
/// # async fn run() -> worker::Result<()> {
/// use worker::fetch;
///
/// let req = http::Request::get("https://www.rust-lang.org/")
///     .body(())
///     .unwrap();
///
/// let res = fetch(req)
///     .await?
///     .into_body()
///     .text()
///     .await?;
///
/// println!("{res}");
/// # Ok(())
/// # }
/// ```
pub async fn fetch(req: http::Request<impl Into<Body>>) -> Result<http::Response<Body>> {
    let fut = {
        let req = req.map(Into::into);
        let global = js_sys::global().unchecked_into::<WorkerGlobalScope>();

        let req = request::into_wasm(req);
        let promise = global.fetch_with_request(&req);

        SendJsFuture::from(promise)
    };

    fut.await
        .map(|res| response::from_wasm(res.unchecked_into()))
        .map_err(Error::from)
}

pub async fn fetch_with_init(
    req: http::Request<impl Into<Body>>,
    init: &RequestInit,
) -> Result<http::Response<Body>> {
    let fut = {
        let req = req.map(Into::into);
        let global = js_sys::global().unchecked_into::<WorkerGlobalScope>();

        let req = request::into_wasm(req);
        let promise = global.fetch_with_request_and_init(&req, &init.into());

        SendJsFuture::from(promise)
    };

    fut.await
        .map(|res| response::from_wasm(res.unchecked_into()))
        .map_err(Error::from)
}

pub async fn fetch_with_signal(
    req: http::Request<impl Into<Body>>,
    signal: &AbortSignal,
) -> Result<http::Response<Body>> {
    let mut init = web_sys::RequestInit::new();
    init.signal(Some(signal.inner()));

    let fut = {
        let req = req.map(Into::into);
        let global = js_sys::global().unchecked_into::<WorkerGlobalScope>();

        let req = request::into_wasm(req);
        let promise = global.fetch_with_request_and_init(&req, &init);

        SendJsFuture::from(promise)
    };

    fut.await
        .map(|res| response::from_wasm(res.unchecked_into()))
        .map_err(Error::from)
}

fn _assert_send() {
    use crate::futures::assert_send_value;
    assert_send_value(fetch(http::Request::new(())));
}
