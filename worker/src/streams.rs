use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{Stream, TryStreamExt};
use js_sys::{BigInt, Uint8Array};
use pin_project::pin_project;
use wasm_bindgen::{JsCast, JsValue};
use wasm_streams::readable::IntoStream;
use web_sys::ReadableStream;
use worker_sys::FixedLengthStream as FixedLengthStreamSys;

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

#[pin_project]
pub struct FixedLengthStream {
    length: u64,
    #[pin]
    bytes_read: u64,
    #[pin]
    inner: Pin<Box<dyn Stream<Item = Result<Vec<u8>>> + 'static>>,
}

impl FixedLengthStream {
    pub fn wrap(stream: impl Stream<Item = Result<Vec<u8>>> + 'static, length: u64) -> Self {
        Self {
            length,
            bytes_read: 0,
            inner: Box::pin(stream),
        }
    }
}

impl Stream for FixedLengthStream {
    type Item = Result<Vec<u8>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let item = if let Some(res) = futures_util::ready!(this.inner.poll_next(cx)) {
            let chunk = match res {
                Ok(chunk) => chunk,
                Err(err) => return Poll::Ready(Some(Err(err))),
            };

            *this.bytes_read += chunk.len() as u64;

            if *this.bytes_read > *this.length {
                let err = Error::from(format!(
                    "fixed length stream had different length than expected (expected {}, got {})",
                    *this.length, *this.bytes_read,
                ));
                Some(Err(err))
            } else {
                Some(Ok(chunk))
            }
        } else if *this.bytes_read != *this.length {
            let err = Error::from(format!(
                "fixed length stream had different length than expected (expected {}, got {})",
                *this.length, *this.bytes_read,
            ));
            Some(Err(err))
        } else {
            None
        };

        Poll::Ready(item)
    }
}

impl From<FixedLengthStream> for FixedLengthStreamSys {
    fn from(stream: FixedLengthStream) -> Self {
        let raw = if stream.length < u32::MAX as u64 {
            FixedLengthStreamSys::new(stream.length as u32).unwrap()
        } else {
            FixedLengthStreamSys::new_big_int(BigInt::from(stream.length)).unwrap()
        };

        let js_stream = stream
            .map_ok(|item| -> Vec<u8> { item })
            .map_ok(|chunk| {
                let array = Uint8Array::new_with_length(chunk.len() as _);
                array.copy_from(&chunk);

                array.into()
            })
            .map_err(JsValue::from);

        let stream: ReadableStream = wasm_streams::ReadableStream::from_stream(js_stream)
            .as_raw()
            .clone()
            .unchecked_into();
        let _ = stream.pipe_to(&raw.writable());

        raw
    }
}
