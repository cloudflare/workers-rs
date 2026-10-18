use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, RefUnwindSafe, UnwindSafe};
use std::pin::Pin;
use std::rc::Rc;

use futures_util::future::{AbortHandle, TryFutureExt, abortable};
use futures_util::stream::{Stream, TryStreamExt};
use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use super::sys;

type JsValueStream = dyn Stream<Item = Result<JsValue, JsValue>>;

#[wasm_bindgen(private)]
pub(crate) struct IntoUnderlyingSource {
    inner: Rc<RefCell<Inner>>,
    pull_handle: Option<AbortHandle>,
}

// SAFETY: `Inner` holds an `Option<Pin<Box<JsValueStream>>>` and uses the
// take-and-replace pattern around every fallible await in `Inner::pull`. On
// panic, the stream is already taken out of the `Option`, leaving the cell in
// a clean `None` state. Subsequent calls fail cleanly via `unwrap_throw`
// rather than observing a torn intermediate state, which upholds the logical
// unwind-safety contract `#[wasm_bindgen]` enforces for exported types.
impl UnwindSafe for IntoUnderlyingSource {}
impl RefUnwindSafe for IntoUnderlyingSource {}

impl IntoUnderlyingSource {
    pub fn new(stream: Box<JsValueStream>) -> Self {
        IntoUnderlyingSource {
            inner: Rc::new(RefCell::new(Inner::new(stream))),
            pull_handle: None,
        }
    }
}

#[allow(clippy::await_holding_refcell_ref)]
#[wasm_bindgen]
impl IntoUnderlyingSource {
    pub fn pull(&mut self, controller: sys::ReadableStreamDefaultController) -> Promise {
        let inner = self.inner.clone();
        let fut = async move {
            // This mutable borrow can never panic, since the ReadableStream always queues
            // each operation on the underlying source.
            let mut inner = inner.try_borrow_mut().unwrap_throw();
            inner.pull(controller).await
        };

        // Allow aborting the future from cancel().
        let (fut, handle) = abortable(fut);
        // Ignore errors from aborting the future.
        let fut = fut.unwrap_or_else(|_| Ok(JsValue::undefined()));

        self.pull_handle = Some(handle);
        // SAFETY: We use the take-and-replace pattern in Inner::pull() to ensure
        // that if a panic occurs, the stream is already taken out of the Option,
        // leaving it in a clean None state. This prevents use of corrupted state
        // after a panic is caught.
        future_to_promise(AssertUnwindSafe(fut))
    }

    pub fn cancel(self) {
        // The stream has been canceled, drop everything.
        drop(self);
    }
}

impl Drop for IntoUnderlyingSource {
    fn drop(&mut self) {
        // Abort the pending pull, if any.
        if let Some(handle) = self.pull_handle.take() {
            handle.abort();
        }
    }
}

struct Inner {
    stream: Option<Pin<Box<JsValueStream>>>,
}

impl Inner {
    fn new(stream: Box<JsValueStream>) -> Self {
        Inner {
            stream: Some(stream.into()),
        }
    }

    async fn pull(
        &mut self,
        controller: sys::ReadableStreamDefaultController,
    ) -> Result<JsValue, JsValue> {
        // Take the stream out before the fallible/panickable operation.
        // This ensures that if a panic occurs, self.stream is already None,
        // so any subsequent call will fail cleanly instead of using corrupted state.
        let mut stream = self.stream.take().unwrap_throw();

        match stream.try_next().await {
            Ok(Some(chunk)) => {
                // Success with chunk: put the stream back and enqueue
                self.stream = Some(stream);
                controller.enqueue_with_chunk(&chunk)?;
            }
            Ok(None) => {
                // Stream closed: don't put it back (it's exhausted), close controller
                controller.close()?;
            }
            Err(err) => {
                // Error: don't put it back, return the error
                return Err(err);
            }
        };
        // Panic: stream is dropped during unwind, self.stream remains None
        Ok(JsValue::undefined())
    }
}
