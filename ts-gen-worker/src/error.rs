use wasm_bindgen::{JsCast, JsValue};

use crate::JsException;

/// The error type every binding in this crate returns through `?`.
///
/// Every caught JS value is classified through `JsException::from`, so a
/// thrown `TypeError` is `Js(JsException::TypeError(_))` whether or not the
/// binding documented it; `@throws` only narrows the binding's own
/// signature to `JsException`. `Rust` carries failures originating on the
/// Rust side.
#[derive(Debug)]
pub enum Error {
    Js(JsException),
    Rust(String),
    #[cfg(feature = "serde")]
    Serde(serde_json::Error),
}

pub type Result<T, E = Error> = core::result::Result<T, E>;

impl From<JsValue> for Error {
    fn from(value: JsValue) -> Self {
        Error::Js(JsException::from(value))
    }
}

impl From<JsException> for Error {
    fn from(error: JsException) -> Self {
        Error::Js(error)
    }
}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Error::Rust(message)
    }
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Error::Rust(message.to_string())
    }
}

#[cfg(feature = "serde")]
impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::Serde(error)
    }
}

#[cfg(feature = "serde")]
impl From<serde_wasm_bindgen::Error> for Error {
    fn from(error: serde_wasm_bindgen::Error) -> Self {
        Error::from(JsValue::from(error))
    }
}

#[cfg(feature = "serde")]
impl From<serde_urlencoded::de::Error> for Error {
    fn from(error: serde_urlencoded::de::Error) -> Self {
        Error::Rust(error.to_string())
    }
}

#[cfg(feature = "url")]
impl From<url::ParseError> for Error {
    fn from(error: url::ParseError) -> Self {
        Error::Rust(error.to_string())
    }
}

impl From<Error> for JsValue {
    fn from(error: Error) -> Self {
        match error {
            Error::Js(error) => error.into(),
            other => js_sys::Error::new(&other.to_string()).into(),
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Js(error) => {
                let value: &JsValue = error.as_ref();
                match value.dyn_ref::<js_sys::Error>() {
                    Some(e) => write!(f, "{}: {}", e.name(), e.message()),
                    None => error.fmt(f),
                }
            }
            Error::Rust(message) => f.write_str(message),
            #[cfg(feature = "serde")]
            Error::Serde(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for Error {}
