#![allow(clippy::new_without_default)]
#![allow(clippy::or_fun_call)]
#![warn(missing_debug_implementations)]
//! # Features
//! ## `d1`
//!
//! Allows the use of [D1 bindings](crate::d1) and [`query!`](crate::query) macro.
//!
//!
//! ## `queue`
//!
//! Enables `queue` event type in [`[event]`](worker_macros::event) macro.
//!
//! ```
//! // Consume messages from a queue
//! #[event(queue)]
//! pub async fn main(message_batch: MessageBatch<MyType>, env: Env, _ctx: Context) -> Result<()> {
//!     Ok(())
//! }
//! ```
//!
//! ## `http`
//! `worker` `0.0.21` introduced an `http` feature flag which starts to replace custom types with widely used types from the [`http`](https://docs.rs/http/latest/http/) crate.
//!
//! This makes it much easier to use crates which use these standard types such as [`axum`].
//!
//! This currently does a few things:
//!
//! 1. Introduce [`Body`], which implements [`http_body::Body`] and is a simple wrapper around [`web_sys::ReadableStream`].
//! 1. The `req` argument when using the [`[event(fetch)]`](worker_macros::event) macro becomes `http::Request<worker::Body>`.
//! 1. The expected return type for the fetch handler is `http::Response<B>` where `B` can be any [`http_body::Body<Data=Bytes>`](http_body::Body).
//! 1. The argument for [`Fetcher::fetch_request`](Fetcher::fetch_request) is `http::Request<worker::Body>`.
//! 1. The return type of [`Fetcher::fetch_request`](Fetcher::fetch_request) is `http::Response<worker::Body>`.
//!
//! The end result is being able to use frameworks like `axum` directly (see [example](./examples/axum)):
//!
//! ```rust
//! pub async fn root() -> &'static str {
//!     "Hello Axum!"
//! }
//!
//! fn router() -> Router {
//!     Router::new().route("/", get(root))
//! }
//!
//! #[event(fetch)]
//! async fn fetch(
//!     req: HttpRequest,
//!     _env: Env,
//!     _ctx: Context,
//! ) -> Result<http::Response<axum::body::Body>> {
//!     Ok(router().call(req).await?)
//! }
//! ```
//!
//! We also implement `try_from` between `worker::Request` and `http::Request<worker::Body>`, and between `worker::Response` and `http::Response<worker::Body>`.
//! This allows you to convert your code incrementally if it is tightly coupled to the original types.
//!
//! ### `Send`
//!
//! All JavaScript value types (`JsValue` and friends) are now `Send` in `wasm-bindgen`,
//! so worker types can be freely shared across async contexts without wrappers.
//!
//! The one exception is `JsFuture`, which still uses `Rc` internally and is not `Send`.
//! The [`send::SendFuture`] wrapper is provided for internal use and for any advanced cases
//! where you need to wrap a `!Send` future in a `Send` context:
//!
//! ```rust
//! let fut = send::SendFuture::new(JsFuture::from(promise));
//! fut.await
//! ```
//!
//! # RPC Support
//! `workers-rs` has experimental support for [Workers RPC](https://developers.cloudflare.com/workers/runtime-apis/rpc/).
//! For now, this relies on JavaScript bindings and may require some manual usage of `wasm-bindgen`.
//!
//! Not all features of RPC are supported yet (or have not been tested), including:
//! - Function arguments and return values
//! - Class instances
//! - Stub forwarding
//!
//! ## RPC Server
//!
//! Writing an RPC server with `workers-rs` is relatively simple. Simply export methods using `wasm-bindgen`. These
//! will be automatically detected by `worker-build` and made available to other Workers. See
//! [example](https://github.com/cloudflare/workers-rs/tree/main/examples/rpc-server).
//!
//! ## RPC Client
//!
//! Creating types and bindings for invoking another Worker's RPC methods is a bit more involved. You will need to
//! write more complex `wasm-bindgen` bindings and some boilerplate to make interacting with the RPC methods more
//! idiomatic. See [example](https://github.com/cloudflare/workers-rs/blob/main/examples/rpc-client/src/calculator.rs).
//!
//! With manually written bindings, it should be possible to support non-primitive argument and return types, using
//! `serde-wasm-bindgen`.
//!
//! ## Generating Client Bindings
//!
//! There are many routes that can be taken to describe RPC interfaces. Under the hood, Workers RPC uses
//! [Cap'N Proto](https://capnproto.org/). A possible future direction is for Wasm guests to include Cap'N Proto
//! serde support and speak directly to the RPC protocol, bypassing JavaScript. This would likely involve defining
//! the RPC interface in Cap'N Proto schema and generating Rust code from that.
//!
//! Another popular interface schema in the WebAssembly community is
//! [WIT](https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md). This is a lightweight format
//! designed for the WebAssembly Component model. `workers-rs` includes an **experimental** code generator which
//! allows you to describe your RPC interface using WIT and generate JavaScript bindings as shown in the
//! [rpc-client example](https://github.com/cloudflare/workers-rs/blob/main/examples/rpc-client/wit/calculator.wit).
//! The easiest way to use this code generator is using a
//! [build script](https://github.com/cloudflare/workers-rs/blob/main/examples/rpc-client/build.rs) as shown in the example.
//! This code generator is pre-alpha, with no support guarantee, and implemented only for primitive types at this time.

#[doc(hidden)]
use std::result::Result as StdResult;

#[doc(hidden)]
pub use async_trait;
pub use js_sys;
pub use url::Url;
pub use wasm_bindgen;
pub use wasm_bindgen_futures;
pub use web_sys;

pub use cf::{Cf, CfResponseProperties, TlsClientAuth};
pub use worker_macros::{consume, durable_object, event, send};
#[doc(hidden)]
pub use worker_sys;
pub use worker_sys::{console_debug, console_error, console_log, console_warn};
pub use worker_sys::{BotManagement, JsDetection};

pub use crate::abort::*;
pub use crate::ai::*;
pub use crate::analytics_engine::*;
pub use crate::cache::{Cache, CacheDeletionOutcome, CacheKey};
pub use crate::container::*;
pub use crate::context::Context;
pub use crate::cors::Cors;
#[cfg(feature = "d1")]
pub use crate::d1::*;
pub use crate::date::{Date, DateInit};
pub use crate::delay::Delay;
pub use crate::durable::*;
pub use crate::dynamic_dispatch::*;
pub use crate::env::{Env, EnvBinding, Secret, Var};
pub use crate::error::Error;
pub use crate::fetcher::Fetcher;
pub use crate::formdata::*;
pub use crate::global::Fetch;
pub use crate::headers::Headers;
pub use crate::http::Method;
pub use crate::hyperdrive::*;
pub use crate::kv::{KvError, KvStore};
#[cfg(feature = "queue")]
pub use crate::queue::*;
pub use crate::r2::*;
pub use crate::rate_limit::{RateLimitOutcome, RateLimiter};
pub use crate::request::{FromRequest, Request};
pub use crate::request_init::*;
pub use crate::response::{EncodeBody, IntoResponse, Response, ResponseBody, ResponseBuilder};
pub use crate::router::{RouteContext, RouteParams, Router};
pub use crate::schedule::*;
pub use crate::secret_store::SecretStore;
pub use crate::socket::*;
pub use crate::streams::*;
pub use crate::version::*;
pub use crate::websocket::*;

mod abort;
mod ai;
mod analytics_engine;
mod cache;
mod cf;
mod container;
mod context;
mod cors;
pub mod crypto;
pub mod panic_abort;
// Require pub module for macro export
#[cfg(feature = "d1")]
/// **Requires** `d1` feature.
pub mod d1;
mod date;
mod delay;
pub mod durable;
mod dynamic_dispatch;
mod env;
mod error;
mod fetcher;
mod formdata;
mod global;
mod headers;
mod http;
mod hyperdrive;
pub mod kv;
#[cfg(feature = "queue")]
mod queue;
mod r2;
mod rate_limit;
mod request;
mod request_init;
mod response;
mod router;
mod schedule;
mod secret_store;
pub mod send;
mod socket;
mod sql;
mod streams;
mod version;
mod websocket;

/// A `Result` alias defaulting to [`Error`].
pub type Result<T, E = error::Error> = StdResult<T, E>;

// This module exists because we can't re-export a standalone `Ok` function at
// the crate root. Doing so would shadow the built-in `Ok` pattern, breaking
// `match` arms like `Ok(x) => ...` when users write `use worker::*`.
/// Use `ok::Ok(value)` as a shorthand for `Result::<_, Error>::Ok(value)`.
pub mod ok {
    use super::*;

    #[allow(non_snake_case)]
    /// Returns `Result::Ok(value)` with `worker::Error` as the error type.
    /// Use as `ok::Ok(value)` rather than importing this function directly.
    pub fn Ok<T>(value: T) -> Result<T> {
        Result::Ok(value)
    }
}

#[cfg(feature = "http")]
/// **Requires** `http` feature. A convenience Body type which wraps [`web_sys::ReadableStream`](web_sys::ReadableStream)
/// and implements [`http_body::Body`](http_body::Body)
pub use http::body::Body;
#[cfg(feature = "http")]
pub use http::{
    request::from_wasm as request_from_wasm, request::to_wasm as request_to_wasm,
    response::from_wasm as response_from_wasm, response::to_wasm as response_to_wasm,
};
#[cfg(feature = "http")]
/// **Requires** `http` feature. Type alias for `http::Request<worker::Body>`.
pub type HttpRequest = ::http::Request<http::body::Body>;
#[cfg(feature = "http")]
/// **Requires** `http` feature. Type alias for `http::Response<worker::Body>`.
pub type HttpResponse = ::http::Response<http::body::Body>;

pub use crate::sql::*;
