//! `Send` wrappers for JS handles. Workers run single-threaded, so a value that is only
//! ever touched on this thread can be marked `Send + Sync` for frameworks (`axum`,
//! `tower`) whose bounds demand it.
//!
//! The `Send`/`Sync` impls here are sound only because `wasm32-unknown-unknown` has no
//! threads: nothing can move the wrapped value or poll the wrapped future from another
//! thread. Building this crate for a target with threads (or wasm with `atomics`)
//! would make them unsound, and the wrappers must not be used with a multi-threaded
//! executor.

use std::fmt::{Debug, Display};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Wraps any future to make it `Send`.
#[derive(Debug)]
#[pin_project::pin_project]
pub struct SendFuture<F> {
    #[pin]
    inner: F,
}

impl<F> SendFuture<F> {
    pub fn new(inner: F) -> Self {
        Self { inner }
    }
}

// SAFETY: single-threaded target; see the module docs.
unsafe impl<F> Send for SendFuture<F> {}
// SAFETY: as above.
unsafe impl<F> Sync for SendFuture<F> {}

impl<F: Future> Future for SendFuture<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

/// `fut.into_send()` for any future.
pub trait IntoSendFuture {
    type Output;
    fn into_send(self) -> SendFuture<Self>
    where
        Self: Sized;
}

impl<F: Future> IntoSendFuture for F {
    type Output = F::Output;

    fn into_send(self) -> SendFuture<Self> {
        SendFuture::new(self)
    }
}

/// Wraps any value to make it `Send + Sync`; derefs to the inner value.
pub struct SendWrapper<T>(pub T);

// SAFETY: single-threaded target; see the module docs.
unsafe impl<T> Send for SendWrapper<T> {}
// SAFETY: as above.
unsafe impl<T> Sync for SendWrapper<T> {}

impl<T> SendWrapper<T> {
    pub fn new(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> std::ops::Deref for SendWrapper<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> std::ops::DerefMut for SendWrapper<T> {
    fn deref_mut(&mut self) -> &mut T {
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
