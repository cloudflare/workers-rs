#![allow(clippy::manual_non_exhaustive)]

pub mod abort;
pub mod cache;
pub mod cf;
pub mod context;
#[cfg(feature = "d1")]
pub mod d1;
pub mod durable_object;
pub mod dynamic_dispatch;
pub mod fetcher;
pub mod file;
pub mod formdata;
pub mod global;
pub mod headers;
#[cfg(feature = "queue")]
pub mod queue;
pub mod r2;
pub mod request;
pub mod request_init;
pub mod response;
pub mod response_init;
pub mod schedule;
pub mod streams;
pub mod websocket;

/// When debugging your Worker via `wrangler dev`, `wrangler tail`, or from the Workers Dashboard,
/// anything passed to this macro will be printed to the terminal or written to the console.
#[macro_export]
macro_rules! console_debug {
    ($($t:tt)*) => (unsafe { $crate::global::debug(&format_args!($($t)*).to_string()) })
}

/// When debugging your Worker via `wrangler dev`, `wrangler tail`, or from the Workers Dashboard,
/// anything passed to this macro will be printed to the terminal or written to the console.
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (unsafe { $crate::global::log(&format_args!($($t)*).to_string()) })
}

/// When debugging your Worker via `wrangler dev`, `wrangler tail`, or from the Workers Dashboard,
/// anything passed to this macro will be printed to the terminal or written to the console.
#[macro_export]
macro_rules! console_warn {
    ($($t:tt)*) => (unsafe { $crate::global::warn(&format_args!($($t)*).to_string()) })
}

/// When debugging your Worker via `wrangler dev`, `wrangler tail`, or from the Workers Dashboard,
/// anything passed to this macro will be printed to the terminal or written to the console.
#[macro_export]
macro_rules! console_error {
    ($($t:tt)*) => (unsafe { $crate::global::error(&format_args!($($t)*).to_string()) })
}

pub mod prelude {
    pub use crate::cf::Cf;
    pub use crate::context::Context;
    #[cfg(feature = "d1")]
    pub use crate::d1::*;
    pub use crate::durable_object;
    pub use crate::file::File;
    pub use crate::formdata::FormData;
    pub use crate::global::{set_timeout, WorkerGlobalScope};
    pub use crate::headers::Headers;
    #[cfg(feature = "queue")]
    pub use crate::queue::*;
    pub use crate::request::Request;
    pub use crate::request_init::*;
    pub use crate::response::Response;
    pub use crate::schedule::*;
    pub use crate::{console_debug, console_error, console_log, console_warn};
}

pub use abort::*;
pub use cf::Cf;
pub use context::Context;
#[cfg(feature = "d1")]
pub use d1::*;
pub use durable_object::*;
pub use dynamic_dispatch::*;
pub use fetcher::*;
pub use file::File;
pub use formdata::FormData;
pub use global::WorkerGlobalScope;
pub use headers::Headers;
#[cfg(feature = "queue")]
pub use queue::*;
pub use r2::*;
pub use request::Request;
pub use request_init::*;
pub use response::Response;
pub use response_init::ResponseInit;
pub use schedule::*;
pub use streams::*;
pub use web_sys::{CloseEvent, ErrorEvent, MessageEvent};
pub use websocket::*;
