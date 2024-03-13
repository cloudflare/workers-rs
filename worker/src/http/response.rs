use super::header::{header_map_from_web_sys_headers, web_sys_headers_from_header_map};
use crate::http::body::Body;
use crate::HttpResponse;
use crate::WebSocket;
use bytes::Bytes;
use futures_util::Stream;
use js_sys::Uint8Array;
use pin_project::pin_project;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use wasm_bindgen::JsValue;
use worker_sys::ext::ResponseExt;
use worker_sys::ext::ResponseInitExt;

#[pin_project]
struct BodyStream<B> {
    #[pin]
    inner: B,
}

impl<B> BodyStream<B> {
    fn new(inner: B) -> Self {
        Self { inner }
    }
}

impl<B: http_body::Body<Data = Bytes>> Stream for BodyStream<B> {
    type Item = Result<JsValue, JsValue>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let inner: Pin<&mut B> = this.inner;
        inner.poll_frame(cx).map(|o| {
            if let Some(r) = o {
                match r {
                    Ok(f) => {
                        if f.is_data() {
                            let b = f.into_data().unwrap();
                            let array = Uint8Array::new_with_length(b.len() as _);
                            array.copy_from(&b);
                            Some(Ok(array.into()))
                        } else {
                            None
                        }
                    }
                    Err(_) => Some(Err(JsValue::from_str("Error polling body"))),
                }
            } else {
                None
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let hint = self.inner.size_hint();
        (hint.lower() as usize, hint.upper().map(|u| u as usize))
    }
}

/// **Requires** `http` feature. Convert generic [`http::Response<B>`](worker::HttpResponse)
/// to [`web_sys::Resopnse`](web_sys::Response) where `B` can be any [`http_body::Body`](http_body::Body)
pub fn to_wasm<B>(mut res: http::Response<B>) -> web_sys::Response
where
    B: http_body::Body<Data = Bytes> + 'static,
{
    let mut init = web_sys::ResponseInit::new();
    init.status(res.status().as_u16());
    init.headers(&web_sys_headers_from_header_map(res.headers()));
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

    web_sys::Response::new_with_opt_readable_stream_and_init(readable_stream.as_ref(), &init)
        .unwrap()
}

/// **Requires** `http` feature. Convert [`web_sys::Resopnse`](web_sys::Response)
/// to [`worker::HttpResponse`](worker::HttpResponse)
pub fn from_wasm(res: web_sys::Response) -> HttpResponse {
    let mut builder =
        http::response::Builder::new().status(http::StatusCode::from_u16(res.status()).unwrap());
    header_map_from_web_sys_headers(res.headers(), builder.headers_mut().unwrap());
    if let Some(ws) = res.websocket() {
        builder = builder.extension(WebSocket::from(ws));
    }
    if let Some(body) = res.body() {
        builder.body(Body::new(body)).unwrap()
    } else {
        builder.body(Body::empty()).unwrap()
    }
}
