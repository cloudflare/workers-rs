#![allow(clippy::new_without_default)]
#![allow(clippy::or_fun_call)]

mod cf;
mod context;
mod cors;
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
mod schedule;
mod streams;
mod websocket;

#[doc(hidden)]
use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, error::Error>;

pub use crate::context::Context;
pub use crate::cors::Cors;
pub use crate::date::{Date, DateInit};
pub use crate::env::Env;
pub use crate::error::Error;
pub use crate::formdata::*;
pub use crate::global::Fetch;
pub use crate::headers::Headers;
pub use crate::http::Method;
pub use crate::request::Request;
pub use crate::request_init::*;
pub use crate::response::{Response, ResponseBody};
pub use crate::router::{RouteContext, RouteParams, Router};
pub use crate::schedule::*;
pub use crate::streams::*;
pub use crate::websocket::*;
pub use cf::Cf;
pub use url::Url;

pub use worker_sys::{console_debug, console_error, console_log, console_warn};

pub use crate::durable::*;
pub use worker_macros::{durable_object, event};

pub use worker_kv as kv;

#[doc(hidden)]
pub use async_trait;
#[doc(hidden)]
pub use js_sys;
#[doc(hidden)]
pub use wasm_bindgen;
#[doc(hidden)]
pub use wasm_bindgen_futures;
#[doc(hidden)]
pub use worker_sys;
