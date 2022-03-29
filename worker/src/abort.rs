use std::ops::Deref;

use wasm_bindgen::JsValue;

use crate::worker_sys::{AbortController as AbortControllerSys, AbortSignal as AbortSignalSys};

/// An interface that allows you to abort in-flight [Fetch](crate::Fetch) requests.
#[derive(Debug)]
pub struct AbortController {
    inner: AbortControllerSys,
}

impl AbortController {
    /// Gets a [AbortSignal] which can be passed to a cancellable operation.
    pub fn signal(&self) -> AbortSignal {
        AbortSignal::from(self.inner.signal())
    }

    /// Aborts any operation using a [AbortSignal] created from this controller.
    pub fn abort(self) {
        self.inner.abort(JsValue::undefined())
    }

    /// Aborts any operation using a [AbortSignal] created from this controller with the provided
    /// reason.
    pub fn abort_with_reason(self, reason: impl Into<JsValue>) {
        self.inner.abort(reason.into())
    }
}

impl Default for AbortController {
    fn default() -> Self {
        Self {
            inner: AbortControllerSys::new(),
        }
    }
}

/// An interface representing a signal that can be passed to cancellable operations, primarily a
/// [Fetch](crate::Fetch) request.
#[derive(Debug)]
pub struct AbortSignal {
    inner: AbortSignalSys,
}

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
        Self::from(AbortSignalSys::abort(JsValue::undefined()))
    }

    /// Creates a [AbortSignal] that is already aborted with the provided reason.
    pub fn abort_with_reason(reason: impl Into<JsValue>) -> Self {
        let reason = reason.into();
        Self::from(AbortSignalSys::abort(reason))
    }
}

impl From<AbortSignalSys> for AbortSignal {
    fn from(inner: AbortSignalSys) -> Self {
        Self { inner }
    }
}

impl Deref for AbortSignal {
    type Target = AbortSignalSys;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
