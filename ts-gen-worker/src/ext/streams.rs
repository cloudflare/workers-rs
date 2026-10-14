use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::{Stream, TryStream, TryStreamExt};
use js_sys::Uint8Array;
use pin_project::pin_project;
use wasm_bindgen::{JsCast, JsValue};
use wasm_streams::readable::IntoStream;

use crate::{Error, ReadableStream, Request, Response, ResponseInitBuilder, Result};

/// A `ReadableStream` of bytes as a Rust `Stream` of chunks; also the body type of
/// `http::Request`/`http::Response` under the `http` feature, where a request without a
/// body is the empty stream.
#[pin_project]
#[derive(Debug)]
pub struct ByteStream {
    #[pin]
    inner: Option<IntoStream<'static>>,
}

impl ByteStream {
    /// A stream that is already at its end.
    pub fn empty() -> Self {
        Self { inner: None }
    }

    /// Back to a JS `ReadableStream`; `None` for the empty stream.
    pub fn into_inner(self) -> Option<ReadableStream> {
        self.inner.map(|stream| {
            wasm_streams::ReadableStream::from_stream(stream)
                .into_raw()
                .unchecked_into()
        })
    }
}

impl From<ReadableStream> for ByteStream {
    fn from(stream: ReadableStream) -> Self {
        Self {
            inner: Some(
                wasm_streams::ReadableStream::from_raw(stream.unchecked_into()).into_stream(),
            ),
        }
    }
}

impl Stream for ByteStream {
    type Item = Result<Vec<u8>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let Some(inner) = this.inner.as_pin_mut() else {
            return Poll::Ready(None);
        };
        let item = match futures_util::ready!(inner.poll_next(cx)) {
            Some(res) => res.map(Uint8Array::from).map_err(Error::from),
            None => return Poll::Ready(None),
        };
        Poll::Ready(match item {
            Ok(value) => Some(Ok(value.to_vec())),
            Err(e) if e.to_string() == "aborted" => None,
            Err(e) => Some(Err(e)),
        })
    }
}

#[cfg(feature = "http")]
impl http_body::Body for ByteStream {
    type Data = bytes::Bytes;
    type Error = Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>>>> {
        self.poll_next(cx)
            .map_ok(|chunk| http_body::Frame::data(bytes::Bytes::from(chunk)))
    }

    fn is_end_stream(&self) -> bool {
        match &self.inner {
            Some(stream) => futures_util::stream::FusedStream::is_terminated(stream),
            None => true,
        }
    }
}

/// Turns a Rust byte stream into a JS `ReadableStream` of `Uint8Array` chunks,
/// the form `BodyInit` names; errors reject the read.
pub fn readable_stream_from<S>(stream: S) -> ReadableStream<Uint8Array>
where
    S: TryStream + 'static,
    S::Ok: Into<Vec<u8>>,
    S::Error: Into<Error>,
{
    let js_stream = stream
        .map_ok(|chunk| JsValue::from(Uint8Array::from(&chunk.into()[..])))
        .map_err(|err| JsValue::from(err.into()));
    wasm_streams::ReadableStream::from_stream(js_stream)
        .into_raw()
        .unchecked_into()
}

impl Request {
    /// The body as a byte stream, if there is one.
    pub fn bytes_stream(&self) -> Option<ByteStream> {
        self.body().map(ByteStream::from)
    }
}

impl Response {
    /// The body as a byte stream, if there is one.
    pub fn bytes_stream(&self) -> Option<ByteStream> {
        self.body().map(ByteStream::from)
    }
}

impl ResponseInitBuilder {
    /// A response whose body is streamed from a Rust `Stream` of byte chunks.
    pub fn from_stream<S>(self, stream: S) -> Result<Response>
    where
        S: TryStream + 'static,
        S::Ok: Into<Vec<u8>>,
        S::Error: Into<Error>,
    {
        Ok(Response::new_with_readable_stream_and_init(
            &readable_stream_from(stream),
            &self.build(),
        )?)
    }
}
