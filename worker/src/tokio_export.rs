//! Handler exports under `worker-build --emscripten --tokio`: the future is
//! scheduled as the root of a fresh Tokio event-loop runtime owned by the
//! invocation, whose wait is the host event loop, so `tokio::spawn`, timers
//! and I/O work inside it without JSPI. Each request gets its own reactor,
//! torn down once the root settles.

use js_sys::Promise;
use std::future::Future;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::tokio::schedule_isolated;

#[doc(hidden)]
pub fn __tokio_promise<F>(future: F) -> Promise
where
    F: Future<Output = Result<JsValue, JsValue>> + 'static,
{
    // The executor runs once, synchronously; a panic in the future is caught
    // by Tokio's task harness and arrives as `Err(JoinError)`.
    let mut future = std::panic::AssertUnwindSafe(Some(future));
    Promise::new(&mut move |resolve, reject| {
        let future = std::mem::take(&mut *future).expect("Promise executor invoked more than once");
        schedule_isolated(future, move |out| {
            let _ = match out {
                Ok(Ok(value)) => resolve.call1(&JsValue::UNDEFINED, &value),
                Ok(Err(err)) => reject.call1(&JsValue::UNDEFINED, &err),
                Err(join) => reject.call1(
                    &JsValue::UNDEFINED,
                    &JsValue::from(format!("handler task failed: {join}")),
                ),
            };
        });
    })
}
