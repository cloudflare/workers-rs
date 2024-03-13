use std::ops::Deref;

use wasm_bindgen::JsValue;
use worker_sys::ext::{AbortControllerExt, AbortSignalExt};

/// An interface that allows you to abort in-flight [Fetch](crate::Fetch) requests.
#[derive(Debug)]
pub struct AbortController {
    inner: web_sys::AbortController,
}

impl AbortController {
    /// Gets a [AbortSignal] which can be passed to a cancellable operation.
    pub fn signal(&self) -> AbortSignal {
        AbortSignal::from(self.inner.signal())
    }

    /// Aborts any operation using a [AbortSignal] created from this controller.
    pub fn abort(self) {
        self.inner.abort()
    }

    /// Aborts any operation using a [AbortSignal] created from this controller with the provided
    /// reason.
    pub fn abort_with_reason(self, reason: impl Into<JsValue>) {
        self.inner.abort_with_reason(&reason.into())
    }
}

impl Default for AbortController {
    fn default() -> Self {
        Self {
            inner: web_sys::AbortController::new().unwrap(),
        }
    }
}

/// An interface representing a signal that can be passed to cancellable operations, primarily a
/// [Fetch](crate::Fetch) request.
#[derive(Debug, Clone)]
pub struct AbortSignal {
    inner: web_sys::AbortSignal,
}

unsafe impl Send for AbortSignal {}
unsafe impl Sync for AbortSignal {}

impl AbortSignal {
    /// A [bool] indicating if the operation that the signal is used for has been aborted.
    pub fn aborted(&self) -> bool {
        self.inner.aborted()
    }

    /// The reason why the signal was aborted.
    pub fn reason(&self) -> Option<JsValue> {
        self.aborted().then(|| self.inner.reason())
    }

    /// Creates a [AbortSignal] that is already aborted.
    pub fn abort() -> Self {
        Self::from(web_sys::AbortSignal::abort())
    }

    /// Creates a [AbortSignal] that is already aborted with the provided reason.
    pub fn abort_with_reason(reason: impl Into<JsValue>) -> Self {
        let reason = reason.into();
        Self::from(web_sys::AbortSignal::abort_with_reason(&reason))
    }

    #[cfg(feature = "http")]
    pub(crate) fn inner(&self) -> &web_sys::AbortSignal {
        &self.inner
    }
}

impl From<web_sys::AbortSignal> for AbortSignal {
    fn from(inner: web_sys::AbortSignal) -> Self {
        Self { inner }
    }
}

impl Deref for AbortSignal {
    type Target = web_sys::AbortSignal;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
