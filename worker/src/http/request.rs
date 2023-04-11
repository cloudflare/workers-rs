//! Functions for translating requests to and from JS

use bytes::Buf;
use futures_util::StreamExt;
use wasm_bindgen::JsCast;
use worker_sys::ext::{HeadersExt, RequestExt};

use crate::{AbortSignal, Cf, CfProperties};

use crate::body::Body;

use super::RequestRedirect;

fn version_from_string(version: &str) -> http::Version {
    match version {
        "HTTP/0.9" => http::Version::HTTP_09,
        "HTTP/1.0" => http::Version::HTTP_10,
        "HTTP/1.1" => http::Version::HTTP_11,
        "HTTP/2" => http::Version::HTTP_2,
        "HTTP/3" => http::Version::HTTP_3,
        _ => unreachable!("no other versions exist"),
    }
}

/// Create a [`http::Request`] from a [`web_sys::Request`].
///
/// # Extensions
///
/// The following types may be added in the [`Extensions`] of the `Request`.
///
/// - [`AbortSignal`]
/// - [`RequestRedirect`]
///
/// # Example
///
/// ```rust,no_run
/// use worker::http::request;
///
/// let req = web_sys::Request::new_with_str("flowers.jpg").unwrap();
/// let req = request::from_wasm(req);
///
/// println!("{} {}", req.method(), req.uri());
/// ```
///
/// [`Extensions`]: http::Extensions
pub fn from_wasm(req: web_sys::Request) -> http::Request<Body> {
    let mut builder = http::Request::builder()
        .method(&*req.method())
        .uri(req.url())
        .extension(AbortSignal::from(req.signal()))
        .extension(RequestRedirect::from(req.redirect()));

    if let Some(cf) = req.cf() {
        builder = builder
            .version(version_from_string(&cf.http_protocol()))
            .extension(Cf::new(cf));
    }

    for header in req.headers().entries() {
        let header = header.unwrap().unchecked_into::<js_sys::Array>();
        builder = builder.header(
            header.get(0).as_string().unwrap(),
            header.get(1).as_string().unwrap(),
        );
    }

    builder.body(Body::from(req)).unwrap()
}

/// Create a [`web_sys::Request`] from a [`http::Request`].
///
/// # Extensions
///
/// The following types may be added in the [`Extensions`] of the `Request`.
///
/// - [`AbortSignal`]
/// - [`CfProperties`]
/// - [`RequestRedirect`]
///
/// # Example
///
/// ```rust,no_run
/// use worker::body::Body;
/// use worker::http::request;
///
/// let req = http::Request::get("https://www.rust-lang.org/")
///     .body(Body::empty())
///     .unwrap();
/// let req = request::into_wasm(req);
///
/// println!("{} {}", req.method(), req.url());
/// ```
///
/// [`Extensions`]: http::Extensions
pub fn into_wasm(mut req: http::Request<Body>) -> web_sys::Request {
    let method = req.method().to_string();
    let uri = req.uri().to_string();

    let headers = web_sys::Headers::new().unwrap();
    for (name, value) in req
        .headers()
        .into_iter()
        .filter_map(|(name, value)| value.to_str().map(|value| (name.as_str(), value)).ok())
    {
        headers.append(name, value).unwrap();
    }

    let mut init = web_sys::RequestInit::new();
    init.method(&method).headers(&headers);

    let signal = req.extensions_mut().remove::<AbortSignal>();
    init.signal(signal.as_ref().map(|s| s.inner()));

    if let Some(redirect) = req.extensions_mut().remove::<RequestRedirect>() {
        init.redirect(redirect.into());
    }

    if let Some(cf) = req.extensions_mut().remove::<CfProperties>() {
        // TODO: this should be handled in worker-sys
        let r = ::js_sys::Reflect::set(
            init.as_ref(),
            &wasm_bindgen::JsValue::from("cf"),
            &wasm_bindgen::JsValue::from(&cf),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
    }

    let body = req.into_body();
    let body = if body.is_none() {
        None
    } else {
        let stream = wasm_streams::ReadableStream::from_stream(body.map(|chunk| {
            chunk
                .map(|buf| js_sys::Uint8Array::from(buf.chunk()).into())
                .map_err(|_| wasm_bindgen::JsValue::NULL)
        }));

        Some(stream.into_raw().unchecked_into())
    };
    init.body(body.as_ref());

    web_sys::Request::new_with_str_and_init(&uri, &init).unwrap()
}
