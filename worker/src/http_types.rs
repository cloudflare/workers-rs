use std::{
    convert::TryFrom,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Buf;
use futures_util::Stream;
use http::{HeaderMap, Request, Response};
use http_body::Body as HttpBody;
use js_sys::Uint8Array;
use wasm_bindgen::{JsCast, JsValue};
use wasm_streams::ReadableStream;

use crate::{
    ByteStream, Cf, Error, Response as WorkerResponse, ResponseBody as WorkerResponseBody,
};

pub use ::http as http_types;

pub struct EdgeRequest(pub worker_sys::Request);

impl TryFrom<EdgeRequest> for crate::HttpRequest {
    type Error = Error;

    fn try_from(e: EdgeRequest) -> Result<Self, Self::Error> {
        let headers: HeaderMap = crate::Headers(e.0.headers()).into();

        let body =
            e.0.body()
                .map(|stream| ReadableStream::from_raw(stream.dyn_into().unwrap()))
                .unwrap_or_else(|| {
                    wasm_streams::ReadableStream::from_stream(futures_util::stream::empty())
                });
        let body = ByteStream {
            inner: body.into_stream(),
        };

        let mut request = Request::new(body);
        *request.headers_mut() = headers;
        *request.method_mut() = http::Method::from_bytes(e.0.method().as_bytes())
            .expect("unable to parse request method");
        *request.uri_mut() = e.0.url().parse().expect("unable to parse url");

        request.extensions_mut().insert(Cf::from(e.0.cf()));

        Ok(request)
    }
}

impl<B> TryFrom<Response<B>> for WorkerResponse
where
    B: HttpBody + 'static,
    B::Error: std::error::Error,
{
    type Error = Error;

    fn try_from(http_response: Response<B>) -> Result<Self, Self::Error> {
        let (parts, body) = http_response.into_parts();

        let body_stream = ReadableStream::from_stream(BodyStream::new(body));
        let resp_body = WorkerResponseBody::Stream(body_stream.into_raw().unchecked_into());

        let resp = WorkerResponse::from_body(resp_body)?
            .with_headers(parts.headers.into())
            .with_status(parts.status.as_u16());

        Ok(resp)
    }
}

#[pin_project::pin_project]
struct BodyStream<B: HttpBody> {
    #[pin]
    inner: B,
    current_data: Option<B::Data>,
}

impl<B> BodyStream<B>
where
    B: HttpBody,
    B::Error: std::error::Error,
{
    pub fn new(body: B) -> Self {
        Self {
            inner: body,
            current_data: None,
        }
    }
}

impl<B> Stream for BodyStream<B>
where
    B: HttpBody + 'static,
    B::Error: std::error::Error,
{
    type Item = Result<JsValue, JsValue>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        if let Some(data) = this.current_data.take() {
            if data.has_remaining() {
                let (data, next_chunk) = advance_data(data);

                // Save it for later usage if there's still data left:
                if data.has_remaining() {
                    *this.current_data = Some(data);
                }

                return Poll::Ready(next_chunk);
            }
        }

        match this.inner.poll_data(cx).map_err(|e| e.to_string().into()) {
            Poll::Ready(Some(Ok(data))) => {
                let (data, next_chunk) = advance_data(data);
                *this.current_data = Some(data);

                Poll::Ready(next_chunk)
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

fn map_slice(slice: &[u8]) -> Option<Result<JsValue, JsValue>> {
    Some(Ok(Uint8Array::from(slice).dyn_into().unwrap()))
}

fn advance_data<D: Buf>(mut data: D) -> (D, Option<Result<JsValue, JsValue>>) {
    let next_chunk = data.chunk();
    let chunk_len = next_chunk.len();
    let next_chunk = map_slice(next_chunk);

    data.advance(chunk_len);

    (data, next_chunk)
}
