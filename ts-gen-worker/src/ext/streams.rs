use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::{Stream, TryStream, TryStreamExt};
use js_sys::Uint8Array;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

use crate::{
    Error, ReadableStream, ReadableStreamDefaultReader, ReadableStreamReadResult, Request,
    Response, ResponseInitBuilder, Result,
};

/// A `ReadableStream` of bytes as a Rust `Stream` of chunks; also the body type of
/// `http::Request`/`http::Response` under the `http` feature, where a request without a
/// body is the empty stream.
///
/// The JS stream is kept as it was given and only locked to a reader on the first poll,
/// so `into_inner()` hands the original object back rather than re-wrapping the Rust side
/// as a new `ReadableStream`. Dropping a `ByteStream` that was polled cancels the stream,
/// as a dropped body should; `into_inner()` only releases the reader's lock, so a
/// consumer can continue from the chunks not yet read.
#[derive(Debug)]
pub struct ByteStream {
    raw: Option<ReadableStream<Uint8Array>>,
    reader: Option<ReadableStreamDefaultReader<Uint8Array>>,
    pending: Option<JsFuture<ReadableStreamReadResult<Uint8Array>, Error>>,
    done: bool,
}

impl ByteStream {
    /// A stream that is already at its end.
    pub fn empty() -> Self {
        Self {
            raw: None,
            reader: None,
            pending: None,
            done: true,
        }
    }

    /// Back to the JS `ReadableStream` this was made from, releasing the reader's lock;
    /// `None` for the empty stream.
    pub fn into_inner(mut self) -> Option<ReadableStream<Uint8Array>> {
        self.pending = None;
        if let Some(reader) = self.reader.take() {
            let _ = reader.release_lock();
        }
        self.raw.take()
    }
}

impl From<ReadableStream<Uint8Array>> for ByteStream {
    fn from(stream: ReadableStream<Uint8Array>) -> Self {
        Self {
            raw: Some(stream),
            reader: None,
            pending: None,
            done: false,
        }
    }
}

impl Drop for ByteStream {
    fn drop(&mut self) {
        if let Some(reader) = self.reader.take() {
            wasm_bindgen_futures::spawn_local(async move {
                let _ = reader.cancel().await;
            });
        }
    }
}

impl ByteStream {
    fn poll_read(&mut self, cx: &mut Context<'_>) -> Poll<Result<Option<Vec<u8>>>> {
        if self.reader.is_none() {
            let Some(raw) = self.raw.as_ref() else {
                return Poll::Ready(Ok(None));
            };
            match raw.get_reader() {
                Ok(reader) => self.reader = Some(reader),
                Err(e) => return Poll::Ready(Err(e.into())),
            }
        }
        let reader = self.reader.as_ref().expect("reader acquired above");
        let pending = match self.pending.as_mut() {
            Some(pending) => pending,
            None => self.pending.insert(JsFuture::from(reader.read())),
        };
        let result = futures_util::ready!(Pin::new(pending).poll(cx));
        self.pending = None;
        Poll::Ready(match result {
            Ok(result) if result.done() => Ok(None),
            Ok(result) => Ok(Some(
                result
                    .value()
                    .map(|chunk| chunk.to_vec())
                    .unwrap_or_default(),
            )),
            Err(e) => Err(e),
        })
    }
}

impl Stream for ByteStream {
    type Item = Result<Vec<u8>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if this.done {
            return Poll::Ready(None);
        }
        let item = match futures_util::ready!(this.poll_read(cx)) {
            Ok(Some(chunk)) => Some(Ok(chunk)),
            Ok(None) => None,
            // The platform rejects pending reads with "aborted" when the other side goes
            // away; that is the end of the stream, not an error.
            Err(e) if e.to_string() == "aborted" => None,
            Err(e) => Some(Err(e)),
        };
        if !matches!(item, Some(Ok(_))) {
            this.done = true;
            this.reader = None;
        }
        Poll::Ready(item)
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
        self.done
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
