use super::header::{header_map_from_web_sys_headers, web_sys_headers_from_header_map};
use crate::http::body::Body;
use crate::HttpResponse;
use crate::Result;
use crate::WebSocket;
use bytes::Bytes;

use crate::http::body::BodyStream;
use worker_sys::ext::ResponseExt;
use worker_sys::ext::ResponseInitExt;

/// **Requires** `http` feature. Convert generic [`http::Response<B>`](crate::HttpResponse)
/// to [`web_sys::Resopnse`](web_sys::Response) where `B` can be any [`http_body::Body`](http_body::Body)
pub fn to_wasm<B>(mut res: http::Response<B>) -> Result<web_sys::Response>
where
    B: http_body::Body<Data = Bytes> + 'static,
{
    let mut init = web_sys::ResponseInit::new();
    init.status(res.status().as_u16());
    let headers = web_sys_headers_from_header_map(res.headers())?;
    init.headers(headers.as_ref());
    if let Some(ws) = res.extensions_mut().remove::<WebSocket>() {
        init.websocket(ws.as_ref());
    }

    let body = res.into_body();
    // I'm not sure how we are supposed to determine if there is no
    // body for an `http::Response`, seems like this may be the only
    // option given the trait? This appears to work for things like
    // `hyper::Empty`.
    let readable_stream = if body.is_end_stream() {
        None
    } else {
        let stream = BodyStream::new(body);
        Some(wasm_streams::ReadableStream::from_stream(stream).into_raw())
    };

    Ok(web_sys::Response::new_with_opt_readable_stream_and_init(
        readable_stream.as_ref(),
        &init,
    )?)
}

/// **Requires** `http` feature. Convert [`web_sys::Response`](web_sys::Response)
/// to [`worker::HttpResponse`](crate::HttpResponse)
pub fn from_wasm(res: web_sys::Response) -> Result<HttpResponse> {
    let mut builder =
        http::response::Builder::new().status(http::StatusCode::from_u16(res.status())?);
    if let Some(headers) = builder.headers_mut() {
        header_map_from_web_sys_headers(res.headers(), headers)?;
    }
    if let Some(ws) = res.websocket() {
        builder = builder.extension(WebSocket::from(ws));
    }
    Ok(if let Some(body) = res.body() {
        builder.body(Body::new(body))?
    } else {
        builder.body(Body::empty())?
    })
}

#[cfg(feature = "axum")]
impl From<crate::Response> for http::Response<axum::body::Body> {
    fn from(resp: crate::Response) -> http::Response<axum::body::Body> {
        let res: web_sys::Response = resp.into();
        let mut builder = http::response::Builder::new()
            .status(http::StatusCode::from_u16(res.status()).unwrap());
        if let Some(headers) = builder.headers_mut() {
            crate::http::header::header_map_from_web_sys_headers(res.headers(), headers).unwrap();
        }
        if let Some(ws) = res.websocket() {
            builder = builder.extension(WebSocket::from(ws));
        }
        if let Some(body) = res.body() {
            builder
                .body(axum::body::Body::new(crate::Body::new(body)))
                .unwrap()
        } else {
            builder
                .body(axum::body::Body::new(crate::Body::empty()))
                .unwrap()
        }
    }
}
