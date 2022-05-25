use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures_util::Stream;
use js_sys::Uint8Array;
use pin_project::pin_project;
use wasm_streams::readable::IntoStream;

use crate::Error;

#[pin_project]
#[derive(Debug)]
pub struct ByteStream {
    #[pin]
    pub(crate) inner: IntoStream<'static>,
}

/// TODO: Definitely safe
unsafe impl Send for ByteStream {}
unsafe impl Sync for ByteStream {}

impl ByteStream {
    pub fn new(inner: IntoStream<'static>) -> Self {
        Self { inner }
    }
}

impl Stream for ByteStream {
    type Item = Result<Vec<u8>, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let item = match futures_util::ready!(this.inner.poll_next(cx)) {
            Some(res) => res.map(Uint8Array::from).map_err(Error::from),
            None => return Poll::Ready(None),
        };

        Poll::Ready(match item {
            Ok(value) => Some(Ok(value.to_vec())),
            Err(e) if e.to_string() == "Error: aborted" => None,
            Err(e) => Some(Err(e)),
        })
    }
}

impl http_body::Body for ByteStream {
    type Data = Bytes;
    type Error = Error;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        self.poll_next(cx).map_ok(Bytes::from)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Result<Option<http::HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }
}
