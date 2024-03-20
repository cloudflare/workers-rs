use futures_util::future::Future;
use pin_project::pin_project;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

#[pin_project]
pub struct SendFuture<F> {
    #[pin]
    inner: F,
}

impl<F> SendFuture<F> {
    pub fn new(inner: F) -> Self {
        Self { inner }
    }
}

unsafe impl<F> Send for SendFuture<F> {}

impl<F: Future> Future for SendFuture<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.inner.poll(cx)
    }
}
