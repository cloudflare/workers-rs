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
pub use crate::cache::{Cache, CacheDeletionOutcome};
pub use crate::context::Context;
pub use crate::date::{Date, DateInit};
pub use crate::delay::Delay;
pub use crate::durable::*;
pub use crate::dynamic_dispatch::*;
pub use crate::env::{Env, Secret, Var};
pub use crate::error::Error;
pub use crate::fetch::fetch;
pub use crate::fetcher::Fetcher;
#[cfg(feature = "queue")]
pub use crate::queue::*;
pub use crate::r2::*;
pub use crate::schedule::*;
pub use crate::streams::*;
pub use crate::websocket::*;

mod abort;
pub mod body;
mod cache;
mod cf;
mod context;
mod date;
mod delay;
pub mod durable;
mod dynamic_dispatch;
mod env;
mod error;
mod fetch;
mod fetcher;
pub mod http;
#[cfg(feature = "queue")]
mod queue;
mod r2;
mod schedule;
mod streams;
mod websocket;

pub type Result<T> = StdResult<T, error::Error>;
