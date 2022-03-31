use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures::Future;
use wasm_bindgen::prelude::Closure;
use worker_sys::global::clear_timeout;

use crate::worker_sys::prelude::set_timeout;

#[pin_project::pin_project(PinnedDrop)]
pub struct Delay {
    inner: Duration,
    closure: Option<Closure<dyn FnMut()>>,
    timeout_id: Option<u32>,
}

impl Future for Delay {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        if this.closure.is_none() {
            let callback_ref = this.closure.get_or_insert_with(move || {
                let waker = cx.waker().clone();
                let wake = Box::new(move || waker.wake_by_ref());
                Closure::wrap(wake as _)
            });

            // Then get that closure back and pass it to setTimeout so we can get woken up later.
            let timeout_id = set_timeout(callback_ref, this.inner.as_millis() as u32);
            *this.timeout_id = Some(timeout_id);

            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

impl From<Duration> for Delay {
    fn from(inner: Duration) -> Self {
        Self {
            inner,
            closure: None,
            timeout_id: None,
        }
    }
}

/// SAFETY: If, for whatever reason, the delay is dropped before the future is ready JS will invoke
/// a dropped future causing memory safety issues. To avoid this we will just clean up the timeout
/// if we drop the delay, cancelling the timeout.
#[pin_project::pinned_drop]
impl PinnedDrop for Delay {
    fn drop(self: Pin<&'_ mut Self>) {
        if let Some(id) = self.project().timeout_id {
            clear_timeout(*id);
        }
    }
}
