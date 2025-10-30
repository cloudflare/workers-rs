//! This module provides utilities for wrapping `!Send` futures
//! in contexts where `Send` is required.
//! Workers is guaranteed to be single-threaded, so it is safe to
//! wrap any future with `Send`.

use futures_util::future::Future;
use pin_project::pin_project;
use std::fmt::Debug;
use std::fmt::Display;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

#[derive(Debug)]
#[pin_project]
/// Wrap any future to make it `Send`.
///
/// ```rust
/// let fut = SendFuture::new(JsFuture::from(promise));
/// fut.await
/// ```
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

/// Trait for SendFuture. Implemented for any type that implements Future.
///
/// ```rust
/// let fut = JsFuture::from(promise).into_send();
/// fut.await
/// ```
pub trait IntoSendFuture {
    type Output;
    fn into_send(self) -> SendFuture<Self>
    where
        Self: Sized;
}

impl<F, T> IntoSendFuture for F
where
    F: Future<Output = T>,
{
    type Output = T;
    fn into_send(self) -> SendFuture<Self> {
        SendFuture::new(self)
    }
}

/// Deprecated: `JsValue` types are now `Send` in `wasm-bindgen`, so `SendWrapper` is no longer
/// needed. Simply use the inner type directly.
#[deprecated(
    since = "0.8.0",
    note = "JsValue types are now Send in wasm-bindgen. Use the inner type directly."
)]
pub struct SendWrapper<T>(pub T);

#[allow(deprecated)]
unsafe impl<T> Send for SendWrapper<T> {}
#[allow(deprecated)]
unsafe impl<T> Sync for SendWrapper<T> {}

#[allow(deprecated)]
impl<T> SendWrapper<T> {
    pub fn new(inner: T) -> Self {
        Self(inner)
    }
}

#[allow(deprecated)]
impl<T> std::ops::Deref for SendWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(deprecated)]
impl<T> std::ops::DerefMut for SendWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[allow(deprecated)]
impl<T: Debug> Debug for SendWrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SendWrapper({:?})", self.0)
    }
}

#[allow(deprecated)]
impl<T: Clone> Clone for SendWrapper<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[allow(deprecated)]
impl<T: Default> Default for SendWrapper<T> {
    fn default() -> Self {
        Self(T::default())
    }
}

#[allow(deprecated)]
impl<T: Display> Display for SendWrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SendWrapper({})", self.0)
    }
}
