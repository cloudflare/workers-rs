//! Task spawning and management.
//!
//! This module provides tokio-compatible task spawning APIs that work
//! in the Cloudflare Workers WASM environment.

use std::{
    cell::RefCell,
    fmt,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

use wasm_bindgen_futures::spawn_local as wasm_spawn_local;

/// Error returned from awaiting a [`JoinHandle`].
///
/// In the WASM environment, this error is returned when a task is aborted.
/// Unlike tokio, panics in spawned tasks will propagate rather than being
/// captured in a `JoinError`.
#[derive(Debug)]
pub struct JoinError {
    cancelled: bool,
}

impl JoinError {
    fn cancelled() -> Self {
        Self { cancelled: true }
    }

    /// Returns `true` if the task was cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    /// Returns `true` if the task panicked.
    ///
    /// Note: In WASM, panics propagate rather than being captured,
    /// so this always returns `false`.
    pub fn is_panic(&self) -> bool {
        false
    }
}

impl fmt::Display for JoinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.cancelled {
            write!(f, "task was cancelled")
        } else {
            write!(f, "task failed")
        }
    }
}

impl std::error::Error for JoinError {}

/// Internal state shared between the spawned task and its [`JoinHandle`].
struct JoinState<T> {
    /// The result of the task, if completed.
    result: Option<Result<T, JoinError>>,
    /// Waker to notify when the task completes.
    waker: Option<Waker>,
    /// Whether the task has been aborted.
    aborted: bool,
}

/// A handle to a spawned task.
///
/// This can be awaited to retrieve the task's return value.
///
/// # Example
///
/// ```rust,ignore
/// use tokio_workers::spawn;
///
/// let handle = spawn(async {
///     // do some work
///     42
/// });
///
/// let result = handle.await.unwrap();
/// assert_eq!(result, 42);
/// ```
pub struct JoinHandle<T> {
    state: Rc<RefCell<JoinState<T>>>,
}

impl<T> JoinHandle<T> {
    /// Abort the task.
    ///
    /// Note: In WASM, this sets a cancellation flag but cannot interrupt
    /// code that is currently executing. The task will be marked as cancelled
    /// the next time it yields.
    pub fn abort(&self) {
        let mut state = self.state.borrow_mut();
        state.aborted = true;
        // If we have a result already, mark it as cancelled
        if state.result.is_none() {
            state.result = Some(Err(JoinError::cancelled()));
            if let Some(waker) = state.waker.take() {
                waker.wake();
            }
        }
    }

    /// Returns `true` if the task has completed.
    pub fn is_finished(&self) -> bool {
        self.state.borrow().result.is_some()
    }
}

impl<T: 'static> Future for JoinHandle<T> {
    type Output = Result<T, JoinError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.borrow_mut();
        if let Some(result) = state.result.take() {
            Poll::Ready(result)
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

/// Spawn a future onto the Workers runtime.
///
/// This function spawns the given future and returns a [`JoinHandle`] that
/// can be awaited to retrieve the result.
///
/// # Differences from tokio
///
/// - No `Send` bound required (WASM is single-threaded)
/// - Panics in spawned tasks propagate rather than being captured
///
/// # Example
///
/// ```rust,ignore
/// use tokio_workers::spawn;
///
/// async fn example() {
///     let handle = spawn(async {
///         // perform some async work
///         "hello"
///     });
///
///     let result = handle.await.unwrap();
///     assert_eq!(result, "hello");
/// }
/// ```
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + 'static,
    F::Output: 'static,
{
    let state = Rc::new(RefCell::new(JoinState {
        result: None,
        waker: None,
        aborted: false,
    }));

    let state_clone = state.clone();
    wasm_spawn_local(async move {
        // Check if aborted before starting
        if state_clone.borrow().aborted {
            return;
        }

        let result = future.await;

        let mut s = state_clone.borrow_mut();
        // Only set result if not already aborted
        if s.result.is_none() {
            s.result = Some(Ok(result));
            if let Some(waker) = s.waker.take() {
                waker.wake();
            }
        }
    });

    JoinHandle { state }
}

/// Spawn a `!Send` future onto the Workers runtime.
///
/// In WASM, this is identical to [`spawn`] since everything runs on a single thread.
///
/// # Example
///
/// ```rust,ignore
/// use tokio_workers::spawn_local;
/// use std::rc::Rc;
///
/// async fn example() {
///     // Rc is !Send, but that's fine in WASM
///     let data = Rc::new(42);
///     let handle = spawn_local(async move {
///         *data
///     });
///
///     let result = handle.await.unwrap();
///     assert_eq!(result, 42);
/// }
/// ```
pub fn spawn_local<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + 'static,
    F::Output: 'static,
{
    spawn(future)
}

/// Runs a blocking function.
///
/// # Behavior
///
/// The behavior depends on which feature flag is enabled:
///
/// - `spawn-blocking-panic` (default): Panics with an error message explaining
///   that blocking operations aren't supported in WASM.
/// - `spawn-blocking-sync`: Runs the closure synchronously. **Warning**: This
///   blocks the event loop and should only be used for very fast operations.
///
/// # Example
///
/// ```rust,ignore
/// use tokio_workers::task::spawn_blocking;
///
/// // With spawn-blocking-sync feature:
/// let handle = spawn_blocking(|| {
///     // This runs synchronously
///     expensive_computation()
/// });
///
/// let result = handle.await.unwrap();
/// ```
#[cfg(any(feature = "spawn-blocking-panic", not(feature = "spawn-blocking-sync")))]
pub fn spawn_blocking<F, R>(_f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + 'static,
    R: 'static,
{
    panic!(
        "spawn_blocking is not supported in WASM. \
         Blocking operations cannot run in the background on a single-threaded runtime. \
         Consider using async alternatives, or enable the 'spawn-blocking-sync' feature \
         to run the closure synchronously (warning: this blocks the event loop)."
    );
}

#[cfg(all(feature = "spawn-blocking-sync", not(feature = "spawn-blocking-panic")))]
pub fn spawn_blocking<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + 'static,
    R: 'static,
{
    // Run synchronously - this blocks the event loop
    let result = f();

    let state = Rc::new(RefCell::new(JoinState {
        result: Some(Ok(result)),
        waker: None,
        aborted: false,
    }));

    JoinHandle { state }
}

/// Yields execution back to the runtime.
///
/// This allows other tasks to run before continuing. Useful in long-running
/// computations to avoid blocking the event loop.
///
/// # Example
///
/// ```rust,ignore
/// use tokio_workers::task::yield_now;
///
/// async fn long_computation() {
///     for i in 0..1000 {
///         // Do some work
///         process_item(i);
///
///         // Periodically yield to let other tasks run
///         if i % 100 == 0 {
///             yield_now().await;
///         }
///     }
/// }
/// ```
pub async fn yield_now() {
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::JsFuture;

    let promise = js_sys::Promise::resolve(&JsValue::UNDEFINED);
    let _ = JsFuture::from(promise).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    async fn test_spawn_returns_value() {
        let handle = spawn(async { 42 });
        let result = handle.await.unwrap();
        assert_eq!(result, 42);
    }

    #[wasm_bindgen_test]
    async fn test_spawn_local_returns_value() {
        let handle = spawn_local(async { "hello" });
        let result = handle.await.unwrap();
        assert_eq!(result, "hello");
    }

    #[wasm_bindgen_test]
    async fn test_spawn_with_rc() {
        use std::rc::Rc;

        let data = Rc::new(42);
        let handle = spawn_local(async move { *data });
        let result = handle.await.unwrap();
        assert_eq!(result, 42);
    }

    #[wasm_bindgen_test]
    async fn test_is_finished() {
        let handle = spawn(async { 42 });

        // Wait for the task to complete
        crate::time::sleep(std::time::Duration::from_millis(10)).await;

        assert!(handle.is_finished());
    }

    #[wasm_bindgen_test]
    async fn test_abort() {
        let handle = spawn(async {
            crate::time::sleep(std::time::Duration::from_millis(100)).await;
            42
        });

        handle.abort();

        let result = handle.await;
        assert!(result.is_err());
        assert!(result.unwrap_err().is_cancelled());
    }

    #[wasm_bindgen_test]
    async fn test_yield_now() {
        yield_now().await;
        // Just verify it doesn't panic
    }

    #[wasm_bindgen_test]
    async fn test_multiple_spawns() {
        let h1 = spawn(async { 1 });
        let h2 = spawn(async { 2 });
        let h3 = spawn(async { 3 });

        let (r1, r2, r3) = futures_util::join!(h1, h2, h3);
        assert_eq!(r1.unwrap(), 1);
        assert_eq!(r2.unwrap(), 2);
        assert_eq!(r3.unwrap(), 3);
    }
}
