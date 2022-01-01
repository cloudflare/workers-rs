pub mod cf;
pub mod durable_object;
pub mod file;
pub mod formdata;
pub mod global;
pub mod headers;
pub mod request;
pub mod request_init;
pub mod response;
pub mod schedule;

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
    pub use crate::durable_object;
    pub use crate::file::File;
    pub use crate::formdata::FormData;
    pub use crate::global::WorkerGlobalScope;
    pub use crate::headers::Headers;
    pub use crate::request::Request;
    pub use crate::request_init::*;
    pub use crate::response::Response;
    pub use crate::schedule::ScheduledEvent;
    pub use crate::{console_debug, console_error, console_log, console_warn};
}

pub use cf::Cf;
pub use durable_object::*;
pub use file::File;
pub use formdata::FormData;
pub use global::WorkerGlobalScope;
pub use headers::Headers;
pub use request::Request;
pub use request_init::*;
pub use response::Response;
pub use web_sys::ResponseInit;
pub use schedule::ScheduledEvent;
