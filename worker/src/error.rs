use wasm_bindgen::{JsCast, JsValue};

/// All possible Error variants that might be encountered while working with a Worker.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    BadEncoding,
    BodyUsed,
    Json((String, u16)),
    JsError(String),
    Internal(JsValue),
    BindingError(String),
    RouteInsertError(matchit::InsertError),
    RouteNoDataError,
    RustError(String),
    SerdeJsonError(serde_json::Error),
    #[cfg(feature = "queue")]
    SerdeWasmBindgenError(serde_wasm_bindgen::Error),
}

impl From<worker_kv::KvError> for Error {
    fn from(e: worker_kv::KvError) -> Self {
        let val: JsValue = e.into();
        val.into()
    }
}

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Self::RustError(e.to_string())
    }
}

impl From<serde_wasm_bindgen::Error> for Error {
    fn from(e: serde_wasm_bindgen::Error) -> Self {
        let val: JsValue = e.into();
        val.into()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BadEncoding => write!(f, "content-type mismatch"),
            Error::BodyUsed => write!(f, "body has already been read"),
            Error::Json((msg, status)) => write!(f, "{msg} (status: {status})"),
            Error::JsError(s) | Error::RustError(s) => {
                write!(f, "{s}")
            }
            Error::Internal(_) => write!(f, "unrecognized JavaScript object"),
            Error::BindingError(name) => write!(f, "no binding found for `{name}`"),
            Error::RouteInsertError(e) => write!(f, "failed to insert route: {e}"),
            Error::RouteNoDataError => write!(f, "route has no corresponding shared data"),
            Error::SerdeJsonError(e) => write!(f, "Serde Error: {e}"),
            #[cfg(feature = "queue")]
            Error::SerdeWasmBindgenError(e) => write!(f, "Serde Error: {e}"),
        }
    }
}

impl std::error::Error for Error {}

// Not sure if the changes I've made here are good or bad...
impl From<JsValue> for Error {
    fn from(v: JsValue) -> Self {
        match v.as_string().or_else(|| {
            v.dyn_ref::<js_sys::Error>().map(|e| {
                format!(
                    "Error: {} - Cause: {}",
                    e.to_string(),
                    e.cause()
                        .as_string()
                        .or_else(|| { Some(e.to_string().into()) })
                        .unwrap_or(String::from("N/A"))
                )
            })
        }) {
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
        Error::RustError(a.to_string())
    }
}

impl From<String> for Error {
    fn from(a: String) -> Self {
        Error::RustError(a)
    }
}

impl From<matchit::InsertError> for Error {
    fn from(e: matchit::InsertError) -> Self {
        Error::RouteInsertError(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::SerdeJsonError(e)
    }
}
