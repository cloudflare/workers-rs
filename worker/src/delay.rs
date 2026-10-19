use std::{
    cell::Cell,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
    time::Duration,
};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    // `setTimeout` returns a number on the web, but with the `nodejs_compat`
    // global timers (the default from compatibility date 2026-08-04) it returns
    // a Node.js-style `Timeout` object. Keep the handle opaque: the type-checked
    // glue of debug builds rejects an object where a number is declared, and
    // `clearTimeout` accepts whatever `setTimeout` returned.
    #[wasm_bindgen(js_name = setTimeout)]
    fn set_timeout(handler: &js_sys::Function, timeout: i32) -> JsValue;

    #[wasm_bindgen(js_name = clearTimeout)]
    fn clear_timeout(handle: &JsValue);
}

/// A [Future] for asynchronously waiting.
///
/// # Example:
/// ```rust,ignore
/// use std::time::Duration;
/// use worker::Delay;
///
/// let duration = Duration::from_millis(1000);
///
/// // Waits a second
/// Delay::from(duration).await;
/// ```
#[derive(Debug)]
#[pin_project::pin_project(PinnedDrop)]
pub struct Delay {
    inner: Duration,
    closure: Option<Closure<dyn FnMut()>>,
    timeout_handle: Option<JsValue>,
    awoken: Rc<Cell<bool>>,
}

impl Future for Delay {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        if !this.awoken.get() {
            if this.closure.is_none() {
                let awoken = this.awoken.clone();
                let callback_ref = this.closure.get_or_insert_with(move || {
                    let waker = cx.waker().clone();
                    let wake = Box::new(move || {
                        waker.wake_by_ref();
                        awoken.set(true);
                    });

                    Closure::wrap_assert_unwind_safe(wake as _)
                });

                // Then get that closure back and pass it to setTimeout so we can get woken up later.
                let handle = set_timeout(
                    callback_ref.as_ref().unchecked_ref::<js_sys::Function>(),
                    this.inner.as_millis() as i32,
                );
                *this.timeout_handle = Some(handle);
            }

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
            timeout_handle: None,
            awoken: Rc::new(Cell::default()),
        }
    }
}

/// SAFETY: If, for whatever reason, the delay is dropped before the future is ready JS will invoke
/// a dropped future causing memory safety issues. To avoid this we will just clean up the timeout
/// if we drop the delay, cancelling the timeout.
#[pin_project::pinned_drop]
impl PinnedDrop for Delay {
    fn drop(self: Pin<&'_ mut Self>) {
        let this = self.project();

        // If we've already completed the future we don't need to clear the timeout.
        if this.awoken.get() {
            return;
        }

        if let Some(handle) = this.timeout_handle.take() {
            clear_timeout(&handle);
        }
    }
}
