use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::{Stream, TryStream, TryStreamExt};
use js_sys::Uint8Array;
use pin_project::pin_project;
use wasm_bindgen::{JsCast, JsValue};
use wasm_streams::readable::IntoStream;

use crate::{Error, ReadableStream, Request, Response, ResponseInitBuilder, Result};

/// A `ReadableStream` of bytes as a Rust `Stream` of chunks.
#[pin_project]
#[derive(Debug)]
pub struct ByteStream {
    #[pin]
    inner: IntoStream<'static>,
}

impl From<ReadableStream> for ByteStream {
    fn from(stream: ReadableStream) -> Self {
        Self {
            inner: wasm_streams::ReadableStream::from_raw(stream.unchecked_into()).into_stream(),
        }
    }
}

impl Stream for ByteStream {
    type Item = Result<Vec<u8>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let item = match futures_util::ready!(this.inner.poll_next(cx)) {
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

/// Turns a Rust byte stream into a JS `ReadableStream`; errors reject the read.
pub fn readable_stream_from<S>(stream: S) -> ReadableStream
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
