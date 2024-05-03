use super::header::{header_map_from_web_sys_headers, web_sys_headers_from_header_map};
use crate::http::body::Body;
use crate::HttpResponse;
use crate::Result;
use crate::WebSocket;
use bytes::Bytes;
use http_body::Body as HttpBody;

use crate::http::body::BodyStream;
use js_sys::Uint8Array;
use worker_sys::ext::ResponseExt;
use worker_sys::ext::ResponseInitExt;

use crate::Error;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;
use std::task::Wake;

struct NoopWaker;

impl Wake for NoopWaker {
    // Required method
    fn wake(self: Arc<Self>) {}
}

/// **Requires** `http` feature. Convert generic [`http::Response<B>`](crate::HttpResponse)
/// to [`web_sys::Resopnse`](web_sys::Response) where `B` can be any [`http_body::Body`](http_body::Body)
pub fn to_wasm<B>(mut res: http::Response<B>) -> Result<web_sys::Response>
where
    B: http_body::Body<Data = Bytes> + Unpin + 'static,
{
    let mut init = web_sys::ResponseInit::new();
    init.status(res.status().as_u16());
    let headers = web_sys_headers_from_header_map(res.headers())?;
    init.headers(headers.as_ref());
    if let Some(ws) = res.extensions_mut().remove::<WebSocket>() {
        init.websocket(ws.as_ref());
    }

    let mut body = res.into_body();

    if let Some(body_size) = body.size_hint().upper() {
        let waker = Arc::new(NoopWaker).into();
        let mut cx = Context::from_waker(&waker);
        let poll = HttpBody::poll_frame(std::pin::Pin::new(&mut body), &mut cx);
        match poll {
            Poll::Ready(Some(Ok(frame))) => {
                // Fixed size body
                let array = Uint8Array::new_with_length(body_size as u32);
                array.copy_from(frame.data_ref().unwrap());
                Ok(web_sys::Response::new_with_opt_buffer_source_and_init(
                    Some(&array),
                    &init,
                )?)
            }
            Poll::Pending => Err(Error::RustError(
                "Unable to poll fixed-length body: Pending".to_owned(),
            )),
            Poll::Ready(None) => Ok(web_sys::Response::new_with_opt_buffer_source_and_init(
                None, &init,
            )?),
            Poll::Ready(Some(Err(_))) => Err(Error::RustError(
                "Unable to poll fixed-length body: Err".to_owned(),
            )),
        }
    } else {
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
