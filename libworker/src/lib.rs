mod cf;
mod date;
pub mod durable;
mod env;
mod error;
mod formdata;
mod global;
mod headers;
mod http;
mod request;
mod request_init;
mod response;
mod router;

use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, error::Error>;

pub use crate::date::{Date, DateInit};
pub use crate::env::Env;
pub use crate::formdata::FormData;
pub use crate::global::Fetch;
pub use crate::headers::Headers;
pub use crate::request::Request;
pub use crate::request_init::*;
pub use crate::response::Response;
pub use crate::router::Router;
pub use cf::Cf;
pub use edgeworker_sys::console_log;
pub use matchit::Params;

pub mod prelude {
    pub use crate::cf::Cf;
    pub use crate::date::{Date, DateInit};
    pub use crate::env::Env;
    pub use crate::formdata::FormData;
    pub use crate::global::Fetch;
    pub use crate::headers::Headers;
    pub use crate::http::Method;
    pub use crate::request::Request;
    pub use crate::request_init::*;
    pub use crate::response::Response;
    pub use crate::router::Router;
    pub use crate::Result;
    pub use edgeworker_sys::console_log;
    pub use matchit::Params;
}
