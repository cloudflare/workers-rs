//! Rust-side ergonomics over the generated bindings: inherent `impl`s on the
//! generated types plus the few Rust-native helper types (`Method`, `ByteStream`).

mod console;
mod context;
mod cors;
mod date;
mod delay;
mod handler;
mod headers;
mod http;
#[cfg(feature = "http")]
mod interop;
mod response;
#[cfg(feature = "serde")]
mod serde;
mod streams;

pub use cors::Cors;
pub use date::{DateExt, DateInit};
pub use delay::Delay;
pub use handler::{FromRequest, IntoResponse};
pub use headers::HeadersIter;
pub use http::Method;
#[cfg(feature = "http")]
pub use interop::{HttpRequest, HttpResponse};
pub use streams::{readable_stream_from, ByteStream};
