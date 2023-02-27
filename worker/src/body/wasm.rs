use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use http::HeaderMap;
use wasm_streams::readable::IntoStream;

pub(crate) struct WasmStreamBody(IntoStream<'static>);

unsafe impl Send for WasmStreamBody {}

impl WasmStreamBody {
    pub(crate) fn new(stream: IntoStream<'static>) -> Self {
        Self(stream)
    }
}

impl http_body::Body for WasmStreamBody {
    type Data = Bytes;
    type Error = ();

    #[inline]
    fn poll_data(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        self.0
            .poll_next_unpin(cx)
            .map_ok(|buf| js_sys::Uint8Array::from(buf).to_vec().into())
            .map_err(|_| ())
    }

    #[inline]
    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }

    #[inline]
    fn size_hint(&self) -> http_body::SizeHint {
        let (lower, upper) = self.0.size_hint();

        let mut hint = http_body::SizeHint::new();
        hint.set_lower(lower as u64);
        if let Some(upper) = upper {
            hint.set_upper(upper as u64);
        }

        hint
    }
}
