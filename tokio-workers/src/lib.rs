//! Tokio compatibility shim for Cloudflare Workers.
//!
//! This crate provides tokio-like APIs that work in the Workers WASM environment.
//! It allows you to run code written for tokio on Cloudflare Workers with minimal
//! modifications.
//!
//! # Usage
//!
//! Replace `use tokio::*` with `use tokio_workers::*`:
//!
//! ```rust,ignore
//! // Before
//! use tokio::{spawn, time::sleep};
//!
//! // After
//! use tokio_workers::{spawn, time::sleep};
//! use std::time::Duration;
//!
//! async fn example() {
//!     let handle = spawn(async { 42 });
//!     sleep(Duration::from_secs(1)).await;
//!     let result = handle.await.unwrap();
//! }
//! ```
//!
//! # Supported APIs
//!
//! ## Task spawning
//! - [`spawn`] - Spawn a future onto the runtime
//! - [`spawn_local`] - Same as `spawn` (WASM is single-threaded)
//! - [`task::spawn_blocking`] - Feature-gated behavior (see below)
//! - [`task::yield_now`] - Yield execution to other tasks
//!
//! ## Time
//! - [`time::sleep`] - Sleep for a duration
//! - [`time::timeout`] - Require a future to complete within a duration
//!
//! ## Synchronization (re-exported from tokio)
//! - [`sync::Mutex`], [`sync::RwLock`], [`sync::Semaphore`]
//! - [`sync::mpsc`], [`sync::oneshot`], [`sync::broadcast`], [`sync::watch`]
//!
//! ## Macros (re-exported from futures)
//! - [`select!`] - Wait on multiple futures
//! - [`join!`] - Wait for all futures to complete
//!
//! # `spawn_blocking` Behavior
//!
//! The `spawn_blocking` function has configurable behavior via feature flags:
//!
//! - `spawn-blocking-panic` (default if neither set): Panics with an error message
//! - `spawn-blocking-sync`: Runs the closure synchronously (blocks event loop)
//!
//! # Limitations
//!
//! - No true multi-threading (WASM is single-threaded)
//! - `JoinHandle::abort()` cannot interrupt running code
//! - `spawn_blocking` cannot truly run blocking code in the background

pub mod sync;
pub mod task;
pub mod time;

// Re-export common items at crate root (mirrors tokio's API)
pub use task::{spawn, spawn_local, JoinError, JoinHandle};

// Re-export macros from futures-util (compatible replacements)
// These are proc-macros that work with any executor
pub use futures_util::select_biased as select;
pub use futures_util::{join, try_join};
