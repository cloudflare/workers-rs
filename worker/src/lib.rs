#![allow(clippy::new_without_default)]
#![allow(clippy::or_fun_call)]

#[doc(hidden)]
use std::result::Result as StdResult;

#[doc(hidden)]
pub use async_trait;
#[doc(hidden)]
pub use js_sys;
pub use url::Url;
#[doc(hidden)]
pub use wasm_bindgen;
#[doc(hidden)]
pub use wasm_bindgen_futures;
pub use worker_kv as kv;

pub use cf::{Cf, TlsClientAuth};
pub use worker_macros::{durable_object, event};
#[doc(hidden)]
pub use worker_sys;
pub use worker_sys::{console_debug, console_error, console_log, console_warn};

pub use crate::abort::*;
pub use crate::cache::{Cache, CacheDeletionOutcome, CacheKey};
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
#[cfg(feature = "queue")]
pub use crate::queue::*;
pub use crate::r2::*;
pub use crate::request::Request;
pub use crate::request_init::*;
pub use crate::response::{Response, ResponseBody};
pub use crate::router::{RouteContext, RouteParams, Router};
pub use crate::schedule::*;
pub use crate::socket::*;
pub use crate::streams::*;
pub use crate::websocket::*;

mod abort;
mod cache;
mod cf;
mod context;
mod cors;
// Require pub module for macro export
#[cfg(feature = "d1")]
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
#[cfg(feature = "queue")]
mod queue;
mod r2;
mod request;
mod request_init;
mod response;
mod router;
mod schedule;
mod socket;
mod streams;
mod websocket;

pub type Result<T> = StdResult<T, error::Error>;

#[cfg(feature = "http")]
pub use http::body::Body;
#[cfg(feature = "http")]
pub use http::{
    request::from_wasm as request_from_wasm, request::to_wasm as request_to_wasm,
    response::from_wasm as response_from_wasm, response::to_wasm as response_to_wasm,
};
#[cfg(feature = "http")]
pub type HttpRequest = ::http::Request<http::body::Body>;
#[cfg(feature = "http")]
pub type HttpResponse = ::http::Response<http::body::Body>;
