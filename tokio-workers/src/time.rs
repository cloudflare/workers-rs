//! Time-related utilities.
//!
//! This module provides tokio-compatible time APIs that work in the
//! Cloudflare Workers WASM environment using JavaScript's `setTimeout`.

use std::{
    cell::Cell,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
    time::Duration,
};

use pin_project::pin_project;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};

/// A future that completes after the specified duration.
///
/// Created by the [`sleep`] function.
#[pin_project(PinnedDrop)]
pub struct Sleep {
    duration: Duration,
    closure: Option<Closure<dyn FnMut()>>,
    timeout_id: Option<JsValue>,
    completed: Rc<Cell<bool>>,
}

impl std::fmt::Debug for Sleep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sleep")
            .field("duration", &self.duration)
            .field("completed", &self.completed.get())
            .finish()
    }
}

// Helper to call setTimeout on any global (works in Node, Browser, and Workers)
fn set_timeout(callback: &js_sys::Function, delay_ms: i32) -> Result<JsValue, JsValue> {
    let global = js_sys::global();
    let set_timeout_fn = js_sys::Reflect::get(&global, &JsValue::from_str("setTimeout"))?;
    let set_timeout_fn: js_sys::Function = set_timeout_fn.unchecked_into();
    set_timeout_fn.call2(&global, callback, &JsValue::from(delay_ms))
}

// Helper to call clearTimeout on any global
fn clear_timeout(timeout_id: &JsValue) {
    let global = js_sys::global();
    if let Ok(clear_timeout_fn) = js_sys::Reflect::get(&global, &JsValue::from_str("clearTimeout"))
    {
        let clear_timeout_fn: js_sys::Function = clear_timeout_fn.unchecked_into();
        let _ = clear_timeout_fn.call1(&global, timeout_id);
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        if this.completed.get() {
            return Poll::Ready(());
        }

        if this.closure.is_none() {
            let completed = this.completed.clone();
            let waker = cx.waker().clone();

            let callback = Closure::wrap(Box::new(move || {
                completed.set(true);
                waker.wake_by_ref();
            }) as Box<dyn FnMut()>);

            let timeout_id = set_timeout(
                callback.as_ref().unchecked_ref(),
                this.duration.as_millis() as i32,
            )
            .expect("setTimeout failed");

            *this.closure = Some(callback);
            *this.timeout_id = Some(timeout_id);
        }

        Poll::Pending
    }
}

#[pin_project::pinned_drop]
impl PinnedDrop for Sleep {
    fn drop(self: Pin<&mut Self>) {
        let this = self.project();
        if !this.completed.get() {
            if let Some(id) = this.timeout_id {
                clear_timeout(id);
            }
        }
    }
}

/// Creates a future that completes after the specified duration.
///
/// This is equivalent to `tokio::time::sleep` but uses JavaScript's
/// `setTimeout` under the hood.
///
/// # Example
///
/// ```rust,ignore
/// use tokio_workers::time::sleep;
/// use std::time::Duration;
///
/// async fn example() {
///     // Sleep for 1 second
///     sleep(Duration::from_secs(1)).await;
///     println!("1 second has passed");
/// }
/// ```
///
/// # Cancellation
///
/// The sleep is cancelled if the future is dropped before completion.
/// The underlying JavaScript timeout will be cleared.
pub fn sleep(duration: Duration) -> Sleep {
    Sleep {
        duration,
        closure: None,
        timeout_id: None,
        completed: Rc::new(Cell::new(false)),
    }
}

/// Error returned when a timeout expires.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Elapsed(());

impl Elapsed {
    fn new() -> Self {
        Self(())
    }
}

impl std::fmt::Display for Elapsed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "deadline has elapsed")
    }
}

impl std::error::Error for Elapsed {}

/// Requires a future to complete before the specified duration has elapsed.
///
/// If the future completes before the timeout, its output is returned.
/// If the timeout elapses first, an [`Elapsed`] error is returned.
///
/// # Example
///
/// ```rust,ignore
/// use tokio_workers::time::{timeout, sleep};
/// use std::time::Duration;
///
/// async fn example() {
///     // This will succeed
///     let result = timeout(Duration::from_secs(1), async { 42 }).await;
///     assert_eq!(result.unwrap(), 42);
///
///     // This will timeout
///     let result = timeout(
///         Duration::from_millis(10),
///         sleep(Duration::from_secs(1))
///     ).await;
///     assert!(result.is_err());
/// }
/// ```
///
/// # Cancellation
///
/// If the timeout is dropped, both the inner future and the sleep are cancelled.
pub async fn timeout<F: Future>(duration: Duration, future: F) -> Result<F::Output, Elapsed> {
    use futures_util::future::{select, Either};
    use std::pin::pin;

    let sleep_fut = pin!(sleep(duration));
    let task_fut = pin!(future);

    match select(task_fut, sleep_fut).await {
        Either::Left((result, _)) => Ok(result),
        Either::Right((_, _)) => Err(Elapsed::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    async fn test_sleep_completes() {
        sleep(Duration::from_millis(10)).await;
        // Just verify it completes without error
    }

    #[wasm_bindgen_test]
    async fn test_sleep_zero_duration() {
        sleep(Duration::ZERO).await;
        // Zero duration should complete immediately (or nearly so)
    }

    #[wasm_bindgen_test]
    async fn test_timeout_success() {
        let result = timeout(Duration::from_millis(100), async { 42 }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[wasm_bindgen_test]
    async fn test_timeout_elapsed() {
        let result = timeout(Duration::from_millis(10), sleep(Duration::from_millis(100))).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "deadline has elapsed");
    }

    #[wasm_bindgen_test]
    async fn test_timeout_with_value() {
        let result = timeout(Duration::from_millis(100), async {
            sleep(Duration::from_millis(10)).await;
            "completed"
        })
        .await;
        assert_eq!(result.unwrap(), "completed");
    }

    #[wasm_bindgen_test]
    async fn test_multiple_sleeps() {
        let start = js_sys::Date::now();
        sleep(Duration::from_millis(10)).await;
        sleep(Duration::from_millis(10)).await;
        let elapsed = js_sys::Date::now() - start;
        // Should take at least 20ms (but allow some slack for timing)
        assert!(elapsed >= 15.0);
    }

    #[wasm_bindgen_test]
    async fn test_concurrent_sleeps() {
        use futures_util::join;

        let start = js_sys::Date::now();
        join!(
            sleep(Duration::from_millis(20)),
            sleep(Duration::from_millis(20)),
            sleep(Duration::from_millis(20)),
        );
        let elapsed = js_sys::Date::now() - start;
        // Concurrent sleeps should complete in ~20ms, not 60ms
        assert!(elapsed < 50.0);
    }
}
