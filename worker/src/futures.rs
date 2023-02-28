use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{Future, FutureExt};
use js_sys::Promise;
use wasm_bindgen_futures::JsFuture;

/// [`JsFuture`] that is explicitely [`Send`].
pub(crate) struct SendJsFuture(JsFuture);

unsafe impl Send for SendJsFuture {}

impl Future for SendJsFuture {
    type Output = <JsFuture as Future>::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.poll_unpin(cx)
    }
}

impl From<Promise> for SendJsFuture {
    fn from(p: Promise) -> Self {
        Self(JsFuture::from(p))
    }
}

#[allow(unused)]
pub(crate) fn assert_send<T: Send>() {}
#[allow(unused)]
pub(crate) fn assert_send_value(_: impl Send) {}
