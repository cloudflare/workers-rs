use std::cell::Cell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use std::time::Duration;

use js_sys::Function;
use wasm_bindgen::prelude::*;

use crate::{clear_timeout, set_timeout_with_ms_delay};

/// A `Future` that resolves after a duration, over `setTimeout`.
///
/// ```ignore
/// Delay::from(Duration::from_millis(100)).await;
/// ```
#[derive(Debug)]
#[pin_project::pin_project(PinnedDrop)]
pub struct Delay {
    duration: Duration,
    closure: Option<Closure<dyn FnMut(JsValue)>>,
    timeout_id: Option<f64>,
    fired: Rc<Cell<bool>>,
}

impl From<Duration> for Delay {
    fn from(duration: Duration) -> Self {
        Self {
            duration,
            closure: None,
            timeout_id: None,
            fired: Rc::new(Cell::new(false)),
        }
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let this = self.project();
        if this.fired.get() {
            return Poll::Ready(());
        }
        if this.closure.is_none() {
            let fired = this.fired.clone();
            let waker = cx.waker().clone();
            let closure = Closure::<dyn FnMut(JsValue)>::new(move |_| {
                fired.set(true);
                waker.wake_by_ref();
            });
            // `setTimeout` with a function and a finite delay cannot throw.
            *this.timeout_id = set_timeout_with_ms_delay(
                Function::closure_ref(&closure),
                this.duration.as_millis() as f64,
            )
            .ok();
            *this.closure = Some(closure);
        }
        Poll::Pending
    }
}

/// Cancels the timer when dropped before firing, so JS never calls into a freed closure.
#[pin_project::pinned_drop]
impl PinnedDrop for Delay {
    fn drop(self: Pin<&mut Self>) {
        let this = self.project();
        if this.fired.get() {
            return;
        }
        if let Some(id) = this.timeout_id {
            let _ = clear_timeout(*id);
        }
    }
}
