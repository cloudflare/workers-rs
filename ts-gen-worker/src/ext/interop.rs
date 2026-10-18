//! `http` crate interop. The generated `Request`/`Response` convert to and from
//! `http::Request`/`http::Response` with `ByteStream` as the body, so a `#[event(fetch)]`
//! handler can be written over the `http` types.
//!
//! What has no `http` equivalent rides in the extensions: `AbortSignal` and
//! `IncomingRequestCfProperties` on a request, `WebSocket` on a response.
//!
//! The conversions back to the platform types are generic over any `http_body::Body`,
//! which is what `#[event(fetch)]` relies on; a `ByteStream` body (the type the other
//! direction produces) is recognised by downcast and its original `ReadableStream` is
//! handed straight back, so a round trip never re-wraps a stream.

use std::any::Any;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use futures_util::Stream;
use http::{HeaderMap, StatusCode, Version};
use http_body::Body;
use js_sys::Uint8Array;
use pin_project::pin_project;

use crate::{
    readable_stream_from, AbortSignal, ByteStream, Error, FromRequest, Headers,
    IncomingRequestCfProperties, IntoResponse, ReadableStream, Request, RequestInit, Response,
    Result, WebSocket,
};

pub type HttpRequest = http::Request<ByteStream>;
pub type HttpResponse = http::Response<ByteStream>;

type BoxError = Box<dyn std::error::Error>;

impl From<http::Error> for Error {
    fn from(error: http::Error) -> Self {
        Error::Rust(error.to_string())
    }
}

impl From<http::status::InvalidStatusCode> for Error {
    fn from(error: http::status::InvalidStatusCode) -> Self {
        Error::Rust(error.to_string())
    }
}

fn version(protocol: &str) -> Version {
    match protocol {
        "HTTP/0.9" => Version::HTTP_09,
        "HTTP/1.0" => Version::HTTP_10,
        "HTTP/2" => Version::HTTP_2,
        "HTTP/3" => Version::HTTP_3,
        _ => Version::HTTP_11,
    }
}

impl TryFrom<Request> for HttpRequest {
    type Error = Error;

    fn try_from(req: Request) -> Result<Self> {
        let mut builder = http::Request::builder()
            .uri(req.url())
            .method(req.method().as_str())
            .extension(req.signal());
        if let Some(headers) = builder.headers_mut() {
            *headers = HeaderMap::try_from(&req.headers())?;
        }
        if let Some(cf) = req.cf() {
            builder = builder.version(version(&cf.http_protocol())).extension(cf);
        }
        let body = req.body().map_or_else(ByteStream::empty, ByteStream::from);
        Ok(builder.body(body)?)
    }
}

impl<B> TryFrom<http::Request<B>> for Request
where
    B: Body<Data = Bytes> + 'static,
    B::Error: Into<BoxError>,
{
    type Error = Error;

    fn try_from(mut req: http::Request<B>) -> Result<Self> {
        let mut init = RequestInit::builder()
            .method(req.method().as_str())
            .headers(&Headers::try_from(req.headers())?);
        if let Some(signal) = req.extensions_mut().remove::<AbortSignal>() {
            init = init.signal(&signal);
        }
        if let Some(cf) = req.extensions_mut().remove::<IncomingRequestCfProperties>() {
            init = init.cf(&cf);
        }
        let uri = req.uri().to_string();
        if let Some(body) = body_stream(req.into_body()) {
            init = init.body(&body);
        }
        Ok(Request::new_with_str_and_init(&uri, &init.build())?)
    }
}

impl TryFrom<Response> for HttpResponse {
    type Error = Error;

    fn try_from(res: Response) -> Result<Self> {
        let mut builder = http::Response::builder().status(StatusCode::from_u16(res.status())?);
        if let Some(headers) = builder.headers_mut() {
            *headers = HeaderMap::try_from(&res.headers())?;
        }
        if let Some(ws) = res.web_socket() {
            builder = builder.extension(ws);
        }
        let body = res.body().map_or_else(ByteStream::empty, ByteStream::from);
        Ok(builder.body(body)?)
    }
}

impl<B> TryFrom<http::Response<B>> for Response
where
    B: Body<Data = Bytes> + 'static,
    B::Error: Into<BoxError>,
{
    type Error = Error;

    fn try_from(mut res: http::Response<B>) -> Result<Self> {
        let mut init = Response::builder()
            .status(res.status().as_u16())
            .headers(&Headers::try_from(res.headers())?);
        if let Some(ws) = res.extensions_mut().remove::<WebSocket>() {
            init = init.web_socket(&ws);
        }
        init.body(body_stream(res.into_body()))
    }
}

impl FromRequest for HttpRequest {
    fn from_request(req: Request) -> Result<Self> {
        req.try_into()
    }
}

impl<B> IntoResponse for http::Response<B>
where
    B: Body<Data = Bytes> + 'static,
    B::Error: Into<BoxError>,
{
    fn into_response(self) -> Result<Response> {
        self.try_into()
    }
}

/// The JS stream for an `http` body: a `ByteStream`'s own `ReadableStream`, a new one
/// over the frames of any other body, `None` when the body is already at its end.
fn body_stream<B>(body: B) -> Option<ReadableStream<Uint8Array>>
where
    B: Body<Data = Bytes> + 'static,
    B::Error: Into<BoxError>,
{
    let mut slot = Some(body);
    if let Some(bytes) = (&mut slot as &mut dyn Any).downcast_mut::<Option<ByteStream>>() {
        return bytes.take()?.into_inner();
    }
    let body = slot?;
    (!body.is_end_stream()).then(|| readable_stream_from(BodyStream { inner: body }))
}

/// An `http_body::Body`'s data frames as a `Stream`; trailers end it.
#[pin_project]
struct BodyStream<B> {
    #[pin]
    inner: B,
}

impl<B> Stream for BodyStream<B>
where
    B: Body<Data = Bytes>,
    B::Error: Into<BoxError>,
{
    type Item = Result<Vec<u8>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let frame = match futures_util::ready!(self.project().inner.poll_frame(cx)) {
            Some(Ok(frame)) => frame,
            Some(Err(e)) => return Poll::Ready(Some(Err(Error::Rust(e.into().to_string())))),
            None => return Poll::Ready(None),
        };
        Poll::Ready(frame.into_data().ok().map(|data| Ok(data.to_vec())))
    }
}
