use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures_util::Stream;
use http::HeaderMap;
use http_body::Body as _;

use crate::Error;

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
pub struct Body(Option<BoxBody>);

impl Body {
    pub fn new<B>(body: B) -> Self
    where
        B: http_body::Body<Data = Bytes> + Send + 'static,
    {
        try_downcast(body)
            .unwrap_or_else(|body| Self(Some(body.map_err(|_| Error::BadEncoding).boxed_unsync())))
    }

    pub fn none() -> Self {
        Self(None)
    }

    pub async fn bytes(self) -> Result<Bytes, Error> {
        super::to_bytes(self).await
    }

    pub async fn text(self) -> Result<String, Error> {
        let bytes = self.bytes().await?;
        String::from_utf8(bytes.to_vec()).map_err(|_| Error::BadEncoding)
    }

    pub(crate) fn is_none(&self) -> bool {
        self.0.is_none()
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
        match &mut self.0 {
            Some(body) => Pin::new(body).poll_data(cx),
            None => Poll::Ready(None),
        }
    }

    #[inline]
    fn poll_trailers(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        match &mut self.0 {
            Some(body) => Pin::new(body).poll_trailers(cx),
            None => Poll::Ready(Ok(None)),
        }
    }

    #[inline]
    fn size_hint(&self) -> http_body::SizeHint {
        match &self.0 {
            Some(body) => body.size_hint(),
            None => http_body::SizeHint::with_exact(0),
        }
    }

    #[inline]
    fn is_end_stream(&self) -> bool {
        match &self.0 {
            Some(body) => body.is_end_stream(),
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
