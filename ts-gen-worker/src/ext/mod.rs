//! Rust-side ergonomics over the generated bindings: inherent `impl`s on the
//! generated types plus the few Rust-native helper types (`Method`, `ByteStream`).

mod context;
mod headers;
mod http;
mod response;
#[cfg(feature = "serde")]
mod serde;
mod streams;

pub use headers::HeadersIter;
pub use http::Method;
pub use streams::{readable_stream_from, ByteStream};
