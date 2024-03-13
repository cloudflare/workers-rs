use std::{
    pin::Pin,
    task::{Context, Poll},
};
use wasm_bindgen::JsCast;
use wasm_streams::readable::IntoStream;

use crate::Error;
use bytes::Bytes;
use futures_util::TryStream;
use futures_util::TryStreamExt;
use futures_util::{stream::FusedStream, Stream, StreamExt};
use http_body::{Body as HttpBody, Frame};
use js_sys::Uint8Array;

#[derive(Debug)]
pub struct Body(Option<IntoStream<'static>>);

unsafe impl Sync for Body {}
unsafe impl Send for Body {}

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

    /// Create a `Body` using a [`Stream`](futures_util::stream::Stream)
    pub fn from_stream<S>(stream: S) -> Result<Self, crate::Error>
    where
        S: TryStream + 'static,
        S::Ok: Into<Vec<u8>>,
        S::Error: std::fmt::Debug,
    {
        let js_stream = stream
            .map_ok(|item| -> Vec<u8> { item.into() })
            .map_ok(|chunk| {
                let array = Uint8Array::new_with_length(chunk.len() as _);
                array.copy_from(&chunk);
                array.into()
            })
            .map_err(|err| crate::Error::RustError(format!("{:?}", err)))
            .map_err(|e| wasm_bindgen::JsValue::from(e.to_string()));

        let stream = wasm_streams::ReadableStream::from_stream(js_stream);
        let stream: web_sys::ReadableStream = stream.into_raw().dyn_into().unwrap();

        Ok(Self::new(stream))
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
            if let Some(r) = o {
                match r {
                    Ok(f) => {
                        if f.is_data() {
                            let b = f.into_data().unwrap();
                            Some(Ok(b))
                        } else {
                            // Not sure how to handle trailers in Stream
                            None
                        }
                    }
                    Err(_) => Some(Err(Error::RustError("Error polling body".to_owned()))),
                }
            } else {
                None
            }
        })
    }
}

#[cfg(feature = "http")]
impl From<axum::body::Body> for Body {
    fn from(value: axum::body::Body) -> Self {
        Body::from_stream(value.into_data_stream()).unwrap()
    }
}
