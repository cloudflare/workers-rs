use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;
use js_sys::Uint8Array;
use pin_project::pin_project;
use wasm_streams::readable::IntoStream;

use crate::{Error, Result};

#[pin_project]
#[derive(Debug)]
pub struct ByteStream {
    #[pin]
    pub(crate) inner: IntoStream<'static>,
}

impl Stream for ByteStream {
    type Item = Result<Vec<u8>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let item = match futures::ready!(this.inner.poll_next(cx)) {
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
