use wasm_bindgen::{JsCast, JsValue};

/// All possible Error variants that might be encountered while working with a Worker.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    BadEncoding,
    BodyUsed,
    Json((String, u16)),
    JsError(String),
    #[cfg(feature = "http")]
    Http(http::Error),
    Infallible,
    Internal(JsValue),
    Io(std::io::Error),
    BindingError(String),
    RouteInsertError(matchit::InsertError),
    RouteNoDataError,
    RustError(String),
    SerdeJsonError(serde_json::Error),
    SerdeWasmBindgenError(serde_wasm_bindgen::Error),
    #[cfg(feature = "http")]
    StatusCode(http::status::InvalidStatusCode),
    #[cfg(feature = "d1")]
    D1(crate::d1::D1Error),
    Utf8Error(std::str::Utf8Error),
}

unsafe impl Sync for Error {}
unsafe impl Send for Error {}

#[cfg(feature = "http")]
impl From<http::Error> for Error {
    fn from(value: http::Error) -> Self {
        Self::Http(value)
    }
}

#[cfg(feature = "http")]
impl From<http::status::InvalidStatusCode> for Error {
    fn from(value: http::status::InvalidStatusCode) -> Self {
        Self::StatusCode(value)
    }
}

#[cfg(feature = "http")]
impl From<http::header::InvalidHeaderName> for Error {
    fn from(value: http::header::InvalidHeaderName) -> Self {
        Self::RustError(format!("Invalid header name: {:?}", value))
    }
}

#[cfg(feature = "http")]
impl From<http::header::InvalidHeaderValue> for Error {
    fn from(value: http::header::InvalidHeaderValue) -> Self {
        Self::RustError(format!("Invalid header value: {:?}", value))
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::Utf8Error(value)
    }
}

impl From<core::convert::Infallible> for Error {
    fn from(_value: core::convert::Infallible) -> Self {
        Error::Infallible
    }
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

impl From<serde_urlencoded::de::Error> for Error {
    fn from(e: serde_urlencoded::de::Error) -> Self {
        Self::RustError(e.to_string())
    }
}

impl From<serde_wasm_bindgen::Error> for Error {
    fn from(e: serde_wasm_bindgen::Error) -> Self {
        let val: JsValue = e.into();
        val.into()
    }
}

#[cfg(feature = "d1")]
impl From<crate::d1::D1Error> for Error {
    fn from(e: crate::d1::D1Error) -> Self {
        Self::D1(e)
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
            #[cfg(feature = "http")]
            Error::Http(e) => write!(f, "http::Error: {e}"),
            Error::Infallible => write!(f, "infallible"),
            Error::Internal(_) => write!(f, "unrecognized JavaScript object"),
            Error::Io(e) => write!(f, "IO Error: {e}"),
            Error::BindingError(name) => write!(f, "no binding found for `{name}`"),
            Error::RouteInsertError(e) => write!(f, "failed to insert route: {e}"),
            Error::RouteNoDataError => write!(f, "route has no corresponding shared data"),
            Error::SerdeJsonError(e) => write!(f, "Serde Error: {e}"),
            Error::SerdeWasmBindgenError(e) => write!(f, "Serde Error: {e}"),
            #[cfg(feature = "http")]
            Error::StatusCode(e) => write!(f, "{e}"),
            #[cfg(feature = "d1")]
            Error::D1(e) => write!(f, "D1: {e:#?}"),
            Error::Utf8Error(e) => write!(f, "{e}"),
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

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
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

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response<axum::body::Body> {
        axum::response::Response::builder()
            .status(500)
            .body("INTERNAL SERVER ERROR".into())
            .unwrap()
    }
}
