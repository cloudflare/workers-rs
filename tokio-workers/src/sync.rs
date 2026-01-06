//! Synchronization primitives.
//!
//! This module re-exports tokio's synchronization primitives, which are
//! runtime-agnostic and work in any async environment including WASM.
//!
//! # Included Primitives
//!
//! ## Locks
//! - [`Mutex`] - An async mutex
//! - [`RwLock`] - An async read-write lock
//! - [`Semaphore`] - Limits concurrent access to a resource
//!
//! ## Notification
//! - [`Notify`] - Notifies waiting tasks
//! - [`Barrier`] - Synchronizes multiple tasks at a barrier point
//!
//! ## Channels
//! - [`mpsc`] - Multi-producer, single-consumer channel
//! - [`oneshot`] - Single-use channel for one value
//! - [`broadcast`] - Multi-producer, multi-consumer broadcast channel
//! - [`watch`] - Single-producer, multi-consumer channel for watching values
//!
//! ## Lazy Initialization
//! - [`OnceCell`] - A cell that can be written to only once
//!
//! # Note on `_timeout` Methods
//!
//! Some tokio sync primitives have methods ending in `_timeout` (e.g.,
//! `Mutex::lock_timeout`). These require the tokio timer infrastructure
//! and will not work in WASM. Use [`crate::time::timeout`] instead:
//!
//! ```rust,ignore
//! use tokio_workers::{sync::Mutex, time::timeout};
//! use std::time::Duration;
//!
//! async fn example(mutex: &Mutex<i32>) {
//!     // Instead of: mutex.lock_timeout(Duration::from_secs(1))
//!     // Use:
//!     let result = timeout(Duration::from_secs(1), mutex.lock()).await;
//!     match result {
//!         Ok(guard) => println!("Got lock: {}", *guard),
//!         Err(_) => println!("Timeout waiting for lock"),
//!     }
//! }
//! ```

// Locks
pub use tokio::sync::{
    AcquireError, OwnedSemaphorePermit, Semaphore, SemaphorePermit, TryAcquireError,
};
pub use tokio::sync::{Mutex, MutexGuard, OwnedMutexGuard, TryLockError};
pub use tokio::sync::{
    OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};

// Notification
pub use tokio::sync::Notify;
pub use tokio::sync::{Barrier, BarrierWaitResult};

// Lazy initialization
pub use tokio::sync::OnceCell;

// Channels
pub use tokio::sync::broadcast;
pub use tokio::sync::mpsc;
pub use tokio::sync::oneshot;
pub use tokio::sync::watch;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    async fn test_mutex() {
        let mutex = Arc::new(Mutex::new(0));

        let m1 = mutex.clone();
        let h1 = crate::spawn(async move {
            let mut guard = m1.lock().await;
            *guard += 1;
        });

        let m2 = mutex.clone();
        let h2 = crate::spawn(async move {
            let mut guard = m2.lock().await;
            *guard += 1;
        });

        h1.await.unwrap();
        h2.await.unwrap();

        assert_eq!(*mutex.lock().await, 2);
    }

    #[wasm_bindgen_test]
    async fn test_oneshot() {
        let (tx, rx) = oneshot::channel();

        crate::spawn(async move {
            tx.send(42).unwrap();
        });

        let value = rx.await.unwrap();
        assert_eq!(value, 42);
    }

    #[wasm_bindgen_test]
    async fn test_mpsc() {
        let (tx, mut rx) = mpsc::channel(10);

        crate::spawn(async move {
            tx.send(1).await.unwrap();
            tx.send(2).await.unwrap();
            tx.send(3).await.unwrap();
        });

        let mut values = vec![];
        while let Some(v) = rx.recv().await {
            values.push(v);
        }

        assert_eq!(values, vec![1, 2, 3]);
    }

    #[wasm_bindgen_test]
    async fn test_notify() {
        let notify = Arc::new(Notify::new());
        let notify2 = notify.clone();

        let handle = crate::spawn(async move {
            notify2.notified().await;
            42
        });

        // Give the task time to start waiting
        crate::time::sleep(std::time::Duration::from_millis(10)).await;

        notify.notify_one();

        let result = handle.await.unwrap();
        assert_eq!(result, 42);
    }
}
