#![allow(clippy::manual_non_exhaustive)]

pub mod ext;
pub mod types;

pub use types::*;

// TODO: remove the re-export of web_sys here
pub use web_sys;

/// When debugging your Worker via `wrangler dev`, `wrangler tail`, or from the Workers Dashboard,
/// anything passed to this macro will be printed to the terminal or written to the console.
#[macro_export]
macro_rules! console_debug {
    ($($t:tt)*) => {
        $crate::web_sys::console::debug_1(&format_args!($($t)*).to_string().into())
    }
}

/// When debugging your Worker via `wrangler dev`, `wrangler tail`, or from the Workers Dashboard,
/// anything passed to this macro will be printed to the terminal or written to the console.
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => {
        $crate::web_sys::console::log_1(&format_args!($($t)*).to_string().into())
    }
}

/// When debugging your Worker via `wrangler dev`, `wrangler tail`, or from the Workers Dashboard,
/// anything passed to this macro will be printed to the terminal or written to the console.
#[macro_export]
macro_rules! console_warn {
    ($($t:tt)*) => {
        $crate::web_sys::console::warn_1(&format_args!($($t)*).to_string().into())
    }
}

/// When debugging your Worker via `wrangler dev`, `wrangler tail`, or from the Workers Dashboard,
/// anything passed to this macro will be printed to the terminal or written to the console.
#[macro_export]
macro_rules! console_error {
    ($($t:tt)*) => {
        $crate::web_sys::console::error_1(&format_args!($($t)*).to_string().into())
    }
}
