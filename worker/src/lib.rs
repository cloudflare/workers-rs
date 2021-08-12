mod date;
pub mod durable;
mod env;
mod error;
mod global;
mod headers;
mod http;
mod request;
mod response;
mod router;

use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, error::Error>;

pub use crate::date::{Date, DateInit};
pub use crate::env::Env;
pub use crate::global::Fetch;
pub use crate::headers::Headers;
pub use crate::request::Request;
pub use crate::response::Response;
pub use crate::router::Router;
pub use edgeworker_ffi::console_log;
pub use matchit::Params;
pub use web_sys::RequestInit;
pub mod prelude {
    pub use crate::date::{Date, DateInit};
    pub use crate::durable;
    pub use crate::env::Env;
    pub use crate::global::Fetch;
    pub use crate::headers::Headers;
    pub use crate::http::Method;
    pub use crate::request::Request;
    pub use crate::response::Response;
    pub use crate::Result;
    pub use edgeworker_ffi::console_log;
    pub use matchit::Params;
    pub use web_sys::RequestInit;
}
