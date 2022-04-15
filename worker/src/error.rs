use crate::Response;
use serde_json::json;
use std::result::Result;
use thiserror::Error;
use wasm_bindgen::{JsCast, JsValue};

/// All possible Error variants that might be encountered while working with a Worker.

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    BadEncoding,
    BodyUsed,
    ResponseError(Response),
    Json((String, u16)),
    JsError(String),
    Internal(JsValue),
    BindingError(String),
    RouteInsertError(#[from] matchit::InsertError),
    RouteNoDataError,
    ParseError(#[from] url::ParseError),
    StringError(String),
    SerdeJsonError(#[from] serde_json::Error),
    RustError(#[from] anyhow::Error),
}

impl From<Error> for Response {
    fn from(e: Error) -> Self {
        match e {
            Error::ResponseError(resp) => resp,
            Error::Json((msg, code)) => Response::from_json(&json!({
                "msg": msg,
                "code": code
            }))
            .map(|r| r.with_status(code))
            .expect("Encoding to JSON failed"),
            _ => Response::error("An error occurred.", 500).expect("Failed to create a 500 error"),
        }
    }
}

impl From<Error> for worker_sys::Response {
    fn from(e: Error) -> Self {
        let r: Response = e.into();
        r.into()
    }
}

impl From<worker_kv::KvError> for Error {
    fn from(e: worker_kv::KvError) -> Self {
        let val: JsValue = e.into();
        val.into()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BadEncoding => write!(f, "content-type mismatch"),
            Error::BodyUsed => write!(f, "body has already been read"),
            Error::Json((msg, status)) => write!(f, "{} (status: {})", msg, status),
            Error::JsError(s) | Error::StringError(s) => write!(f, "{}", s),
            Error::Internal(_) => write!(f, "unrecognized JavaScript object"),
            Error::ParseError(e) => write!(f, "{}", e),
            Error::BindingError(name) => write!(f, "no binding found for `{}`", name),
            Error::RouteInsertError(e) => write!(f, "failed to insert route: {}", e),
            Error::RouteNoDataError => write!(f, "route has no corresponding shared data"),
            Error::SerdeJsonError(e) => write!(f, "Serde Error: {}", e),
            Error::RustError(e) => write!(f, "{}", e),
            Error::ResponseError(e) => write!(f, "{e:?}"),
        }
    }
}

impl From<JsValue> for Error {
    fn from(v: JsValue) -> Self {
        match v
            .as_string()
            .or_else(|| v.dyn_ref::<js_sys::Error>().map(|e| e.to_string().into()))
        {
            Some(s) => Self::JsError(s),
            None => Self::Internal(v),
        }
    }
}

impl From<Error> for JsValue {
    fn from(e: Error) -> Self {
        JsValue::from_str(&e.to_string())
    }
}

impl From<&str> for Error {
    fn from(a: &str) -> Self {
        Error::StringError(a.to_string())
    }
}

impl From<String> for Error {
    fn from(a: String) -> Self {
        Error::StringError(a)
    }
}
