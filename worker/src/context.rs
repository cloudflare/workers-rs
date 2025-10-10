use std::future::Future;

use crate::worker_sys::Context as JsContext;
use crate::Result;

use serde::de::DeserializeOwned;
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
    /// ```no_run
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

    /// Get the props passed to this worker execution context.
    ///
    /// Props provide a way to pass additional configuration to a worker based on the context
    /// in which it was invoked. For example, when your Worker is called by another Worker via
    /// a Service Binding, props can provide information about the calling worker.
    ///
    /// Props are configured in your wrangler.toml when setting up Service Bindings:
    /// ```toml
    /// [[services]]
    /// binding = "MY_SERVICE"
    /// service = "my-worker"
    /// props = { clientId = "frontend", permissions = ["read", "write"] }
    /// ```
    ///
    /// Then deserialize them to your custom type:
    /// ```no_run
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct MyProps {
    ///     clientId: String,
    ///     permissions: Vec<String>,
    /// }
    ///
    /// let props = ctx.props::<MyProps>()?;
    /// ```
    ///
    /// See: <https://developers.cloudflare.com/workers/runtime-apis/context/#props>
    pub fn props<T: DeserializeOwned>(&self) -> Result<T> {
        Ok(serde_wasm_bindgen::from_value(self.inner.props())?)
    }
}

impl AsRef<JsContext> for Context {
    fn as_ref(&self) -> &JsContext {
        &self.inner
    }
}
