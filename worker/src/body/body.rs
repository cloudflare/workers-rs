use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures_util::Stream;
use http::HeaderMap;
use http_body::Body as _;

use crate::{body::wasm::WasmStreamBody, futures::SendJsFuture, Error};

type BoxBody = http_body::combinators::UnsyncBoxBody<Bytes, Error>;

fn try_downcast<T, K>(k: K) -> Result<T, K>
where
    T: 'static,
    K: Send + 'static,
{
    let mut k = Some(k);
    if let Some(k) = <dyn std::any::Any>::downcast_mut::<Option<T>>(&mut k) {
        Ok(k.take().unwrap())
    } else {
        Err(k.unwrap())
    }
}

#[derive(Debug)]
enum BodyInner {
    Regular(BoxBody),
    Request(web_sys::Request),
    Response(web_sys::Response),
}

unsafe impl Send for BodyInner {}

impl BodyInner {
    async fn bytes(self) -> Result<Bytes, Error> {
        async fn array_buffer_to_bytes(
            buf: Result<js_sys::Promise, wasm_bindgen::JsValue>,
        ) -> Result<Bytes, Error> {
            let fut = SendJsFuture::from(buf.map_err(Error::Internal)?);
            let buf = js_sys::Uint8Array::from(fut.await.unwrap());
            Ok(buf.to_vec().into())
        }

        match self {
            BodyInner::Regular(body) => super::to_bytes(body).await,
            BodyInner::Request(req) => array_buffer_to_bytes(req.array_buffer()).await,
            BodyInner::Response(res) => array_buffer_to_bytes(res.array_buffer()).await,
        }
    }
}

#[derive(Debug)]
pub struct Body(Option<BodyInner>);

impl Body {
    pub fn new<B>(body: B) -> Self
    where
        B: http_body::Body<Data = Bytes> + Send + 'static,
    {
        try_downcast(body).unwrap_or_else(|body| {
            Self(Some(BodyInner::Regular(
                body.map_err(|_| Error::BadEncoding).boxed_unsync(),
            )))
        })
    }

    pub fn none() -> Self {
        Self(None)
    }

    pub async fn bytes(self) -> Result<Bytes, Error> {
        match self.0 {
            Some(inner) => inner.bytes().await,
            None => Ok(Bytes::new()),
        }
    }

    pub async fn text(self) -> Result<String, Error> {
        let bytes = self.bytes().await?;
        String::from_utf8(bytes.to_vec()).map_err(|_| Error::BadEncoding)
    }

    pub(crate) fn is_none(&self) -> bool {
        self.0.is_none()
    }

    /// Turns the body into a regular streaming body, if it's not already, and returns the underlying body.
    fn as_inner_box_body(&mut self) -> Option<&mut BoxBody> {
        match &self.0 {
            Some(BodyInner::Request(req)) => *self = req.body().map(WasmStreamBody::new).into(),
            Some(BodyInner::Response(res)) => *self = res.body().map(WasmStreamBody::new).into(),
            _ => {}
        }

        match self.0.as_mut()? {
            BodyInner::Regular(body) => Some(body),
            _ => unreachable!(),
        }
    }
}

impl Default for Body {
    fn default() -> Self {
        Self::none()
    }
}

impl From<()> for Body {
    fn from(_: ()) -> Self {
        Self::none()
    }
}

impl<B> From<Option<B>> for Body
where
    B: http_body::Body<Data = Bytes> + Send + 'static,
{
    fn from(body: Option<B>) -> Self {
        body.map(Body::new).unwrap_or_else(Self::none)
    }
}

impl From<web_sys::Request> for Body {
    fn from(req: web_sys::Request) -> Self {
        Self(req.body().map(|_| BodyInner::Request(req)))
    }
}

impl From<web_sys::Response> for Body {
    fn from(res: web_sys::Response) -> Self {
        Self(res.body().map(|_| BodyInner::Response(res)))
    }
}

macro_rules! body_from_impl {
    ($ty:ty) => {
        impl From<$ty> for Body {
            fn from(buf: $ty) -> Self {
                Self::new(http_body::Full::from(buf))
            }
        }
    };
}

body_from_impl!(&'static [u8]);
body_from_impl!(std::borrow::Cow<'static, [u8]>);
body_from_impl!(Vec<u8>);

body_from_impl!(&'static str);
body_from_impl!(std::borrow::Cow<'static, str>);
body_from_impl!(String);

body_from_impl!(Bytes);

impl http_body::Body for Body {
    type Data = Bytes;
    type Error = Error;

    #[inline]
    fn poll_data(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        match self.as_inner_box_body() {
            Some(body) => Pin::new(body).poll_data(cx),
            None => Poll::Ready(None),
        }
    }

    #[inline]
    fn poll_trailers(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        match self.as_inner_box_body() {
            Some(body) => Pin::new(body).poll_trailers(cx),
            None => Poll::Ready(Ok(None)),
        }
    }

    #[inline]
    fn size_hint(&self) -> http_body::SizeHint {
        match &self.0 {
            Some(BodyInner::Regular(body)) => body.size_hint(),
            Some(BodyInner::Request(_)) => http_body::SizeHint::new(),
            Some(BodyInner::Response(_)) => http_body::SizeHint::new(),
            None => http_body::SizeHint::with_exact(0),
        }
    }

    #[inline]
    fn is_end_stream(&self) -> bool {
        match &self.0 {
            Some(BodyInner::Regular(body)) => body.is_end_stream(),
            Some(BodyInner::Request(_)) => false,
            Some(BodyInner::Response(_)) => false,
            None => true,
        }
    }
}

impl Stream for Body {
    type Item = Result<Bytes, Error>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.poll_data(cx)
    }
}
