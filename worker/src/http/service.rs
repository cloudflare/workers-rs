use std::convert::TryInto;
use std::task::Context;
use std::{future::Future, pin::Pin, task::Poll};

use crate::send::SendFuture;
use crate::{Body, Error, Fetch, HttpResponse, Request};

/// A Tower-compatible Service implementation for Cloudflare Workers.
///
/// This struct implements the `tower::Service` trait, allowing it to be used
/// as a service in the Tower middleware ecosystem.
#[derive(Debug, Default, Clone, Copy)]
pub struct Service;

impl<B: http_body::Body<Data = bytes::Bytes> + Clone + 'static> tower::Service<http::Request<B>>
    for Service
{
    type Response = http::Response<Body>;
    type Error = Error;
    type Future =
        Pin<Box<dyn Future<Output = std::result::Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<B>) -> Self::Future {
        // Convert a http Request to a worker Request
        let worker_request: Request = req.try_into().unwrap();

        // Send the request with Fetch and receive a Future
        let fut = Fetch::Request(worker_request).send();

        // Convert the Future output to a HttpResponse
        let http_response =
            async { Ok(TryInto::<HttpResponse>::try_into(fut.await.unwrap()).unwrap()) };

        // Wrap the Future in a SendFuture to make it Send
        let wrapped = SendFuture::new(http_response);

        Box::pin(wrapped)
    }
}
