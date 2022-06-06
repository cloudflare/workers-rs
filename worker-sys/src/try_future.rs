use futures::task::Poll;
use pin_project::pin_project;
use wasm_bindgen::prelude::*;

use std::future::Future;
use std::pin::Pin;
use std::task::Context;

#[pin_project]
pub struct TryFuture<F: ?Sized> {
    #[pin]
    pub f: F,
}
impl<F: ?Sized> Future for TryFuture<F>
where
    F: Future,
{
    type Output = Result<F::Output, ()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match try__(|| self.project().f.poll(cx)) {
            Ok(Poll::Ready(t)) => Poll::Ready(Ok(t)),
            Ok(Poll::Pending) => Poll::Pending,
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

fn try__<O>(f: impl FnOnce() -> O) -> Result<O, ()> {
    let mut f = Some(f);
    let mut o = None;
    try_(&mut || o = Some(f.take().unwrap()()));
    o.ok_or(())
}

#[wasm_bindgen]
extern "C" {
    fn try_(a: &mut dyn FnMut());
    pub fn new_wasm_instance();
}
