//! HTTP types and functions

mod clone;
mod redirect;
pub mod request;
pub mod response;

pub use clone::HttpClone;
pub use redirect::RequestRedirect;

pub use http::*;
