use std::future::Future;

use crate::worker_sys::Context as JsContext;

use wasm_bindgen::JsValue;
use wasm_bindgen_futures::future_to_promise;

/// A context bound to a `fetch` event.
#[derive(Debug)]
pub struct Context {
    inner: JsContext,
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl Context {
    /// Constructs a context from an underlying JavaScript context object.
    pub fn new(inner: JsContext) -> Self {
        Self { inner }
    }

    /// Extends the lifetime of the "fetch" event which this context is bound to,
    /// until the given future has been completed. The future is executed before the handler
    /// terminates but does not block the response. For example, this is ideal for caching
    /// responses or handling logging.
    /// ```ignore
    /// context.wait_until(async move {
    ///     let _ = cache.put(request, response).await;
    /// });
    /// ```
    pub fn wait_until<F>(&self, future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        self.inner
            .wait_until(&future_to_promise(async {
                future.await;
                Ok(JsValue::UNDEFINED)
            }))
            .unwrap()
    }

    /// Prevents a runtime error response when the Worker script throws an unhandled exception.
    /// Instead, the script will "fail open", which will proxy the request to the origin server
    /// as though the Worker was never invoked.
    pub fn pass_through_on_exception(&self) {
        self.inner.pass_through_on_exception().unwrap()
    }
}

impl AsRef<JsContext> for Context {
    fn as_ref(&self) -> &JsContext {
        &self.inner
    }
}
