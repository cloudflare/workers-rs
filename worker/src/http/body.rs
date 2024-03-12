use std::{
    pin::Pin,
    task::{Context, Poll},
};
use wasm_bindgen::JsCast;
use wasm_streams::readable::IntoStream;

use crate::Error;
use bytes::Bytes;
use futures_util::{stream::FusedStream, Stream, StreamExt};
use http_body::{Body as HttpBody, Frame};

#[derive(Debug)]
pub struct Body(Option<IntoStream<'static>>);

impl Body {
    pub fn new(stream: web_sys::ReadableStream) -> Self {
        Self(Some(
            wasm_streams::ReadableStream::from_raw(stream.unchecked_into()).into_stream(),
        ))
    }

    pub fn into_inner(self) -> Option<web_sys::ReadableStream> {
        self.0
            .map(|s| wasm_streams::ReadableStream::from_stream(s).into_raw())
    }

    pub fn empty() -> Self {
        Self(None)
    }
}

impl HttpBody for Body {
    type Data = Bytes;
    type Error = Error;

    #[inline]
    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        if let Some(ref mut stream) = &mut self.0 {
            stream
                .poll_next_unpin(cx)
                .map_ok(|buf| {
                    let bytes = Bytes::copy_from_slice(&js_sys::Uint8Array::from(buf).to_vec());
                    Frame::data(bytes)
                })
                .map_err(Error::Internal)
        } else {
            Poll::Ready(None)
        }
    }

    #[inline]
    fn size_hint(&self) -> http_body::SizeHint {
        let mut hint = http_body::SizeHint::new();
        if let Some(ref stream) = self.0 {
            let (lower, upper) = stream.size_hint();

            hint.set_lower(lower as u64);
            if let Some(upper) = upper {
                hint.set_upper(upper as u64);
            }
        } else {
            hint.set_lower(0);
            hint.set_upper(0);
        }
        hint
    }

    fn is_end_stream(&self) -> bool {
        if let Some(ref stream) = self.0 {
            stream.is_terminated()
        } else {
            true
        }
    }
}

impl Stream for Body {
    type Item = Result<Bytes, Error>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.poll_frame(cx).map(|o| {
            o.map(|r| match r {
                Ok(f) => match f.into_data() {
                    Ok(b) => Ok(b),
                    Err(_) => Err(Error::RustError("Error polling body".to_owned())),
                },
                Err(_) => Err(Error::RustError("Error polling body".to_owned())),
            })
        })
    }
}