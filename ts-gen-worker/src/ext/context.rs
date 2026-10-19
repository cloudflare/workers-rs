use std::future::Future;

use wasm_bindgen::JsValue;
use wasm_bindgen_futures::future_to_promise;

use crate::ExecutionContext;

impl ExecutionContext {
    /// Runs `future` in the background, keeping the invocation alive until it completes.
    /// The `Future` form of `wait_until`.
    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        self.wait_until(&future_to_promise(async {
            future.await;
            Ok(JsValue::UNDEFINED)
        }))
    }
}
