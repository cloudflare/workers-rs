//! This module provides utilities for working with JavaScript types
//! which do not implement `Send`, in contexts where `Send` is required.
//! Workers is guaranteed to be single-threaded, so it is safe to
//! wrap any type with `Send` and `Sync` traits.

use futures_util::future::Future;
use pin_project::pin_project;
use std::fmt::Debug;
use std::fmt::Display;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

/// Wrap any future to make it `Send`.
///
/// ```no_run
/// # use wasm_bindgen_futures::JsFuture;
/// # use worker::send::SendFuture;
/// # tokio_test::block_on(async {
/// # let promise = js_sys::Promise::new(&mut |_, _| {});
/// let fut = SendFuture::new(JsFuture::from(promise));
/// fut.await;
/// # })
/// ```
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
unsafe impl<F> Sync for SendFuture<F> {}

impl<F: Future> Future for SendFuture<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.inner.poll(cx)
    }
}

/// Wrap any type to make it `Send`.
///
/// ```no_run
/// # use worker::send::SendWrapper;
/// # let promise = js_sys::Promise::new(&mut |_, _| {});
/// /// js_sys::Promise is !Send
/// let send_promise = SendWrapper::new(promise);
/// ```
pub struct SendWrapper<T>(pub T);

unsafe impl<T> Send for SendWrapper<T> {}
unsafe impl<T> Sync for SendWrapper<T> {}

impl<T> SendWrapper<T> {
    pub fn new(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> std::ops::Deref for SendWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for SendWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Debug> Debug for SendWrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SendWrapper({:?})", self.0)
    }
}

impl<T: Clone> Clone for SendWrapper<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Default> Default for SendWrapper<T> {
    fn default() -> Self {
        Self(T::default())
    }
}

impl<T: Display> Display for SendWrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SendWrapper({})", self.0)
    }
}
