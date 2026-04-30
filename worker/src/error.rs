use crate::kv::KvError;
use js_sys::Reflect;
use strum::IntoStaticStr;
use wasm_bindgen::{JsCast, JsValue};

/// All possible Error variants that might be encountered while working with a Worker.
#[derive(Debug, IntoStaticStr)]
#[non_exhaustive]
pub enum Error {
    BadEncoding,
    BodyUsed,
    Json((String, u16)),
    /// Error used for strings thrown from JS
    JsError(String),
    #[cfg(feature = "http")]
    Http(http::Error),
    Infallible,
    /// Error used for unknown JS values thrown from JS
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
    #[cfg(feature = "timezone")]
    TimezoneError,
    KvError(KvError),

    // Email errors
    /// Email recipient is on the suppression list (typically due to prior
    /// bounces or complaints) and cannot be delivered to.
    EmailRecipientSuppressed(String),
    /// Email recipient is not allowed for this sender (policy/restriction).
    ///
    /// Distinct from `EmailRecipientSuppressed`: suppression is a delivery
    /// outcome, "not allowed" is a sender-policy refusal.
    EmailRecipientNotAllowed(String),

    // General errors
    /// Cloudflare rate limit exceeded.
    ///
    /// Currently only produced by the email forwarder. May be produced by other
    /// rate-limited subsystems in the future.
    RateLimitExceeded(String),
    /// Cloudflare daily limit exceeded.
    ///
    /// Currently only produced by the email forwarder. May be produced by other
    /// quota-limited subsystems in the future.
    DailyLimitExceeded(String),
    /// A backend Cloudflare service reported an internal error.
    ///
    /// Generally transient; consider retrying with backoff.
    ///
    /// Currently only produced by the email forwarder. May be produced by other
    /// subsystems in the future.
    InternalError(String),

    /// A JavaScript error that didn't match any structured variant.
    ///
    /// This is the catch-all for errors thrown by the Workers runtime or
    /// user code that don't carry a recognized error code.
    ///
    /// Always produced by converting from a `JsValue`; not intended to be
    /// constructed directly from Rust code. Round-tripping back to `JsValue`
    /// returns the stored `original`, so the original error object identity
    /// (stack, prototype, extra properties) is preserved.
    UnknownJsError {
        /// The original error value.
        original: JsValue,
        /// Cached `name` extracted at conversion time.
        name: Option<String>,
        /// Cached `message` extracted at conversion time.
        message: String,
        /// Cached `code` extracted at conversion time.
        code: Option<String>,
        /// Recursively converted `cause`.
        cause: Option<Box<Error>>,
    },
}

const MAX_CAUSE_DEPTH: u32 = 16;
fn convert_js_error_with_depth(err: js_sys::Error, depth: u32) -> Error {
    let message: String = err.message().into();
    let name = err.name().as_string();
    let code = Reflect::get_str(&err, &"code".into())
        .ok()
        .flatten()
        .and_then(|v| v.as_string());
    if let Some(code_str) = &code {
        match code_str.as_str() {
            "E_RECIPIENT_SUPPRESSED" => return Error::EmailRecipientSuppressed(message),
            "RCPT_NOT_ALLOWED" => return Error::EmailRecipientNotAllowed(message),
            "E_RATE_LIMIT_EXCEEDED" => return Error::RateLimitExceeded(message),
            "E_DAILY_LIMIT_EXCEEDED" => return Error::DailyLimitExceeded(message),
            "E_INTERNAL_SERVER_ERROR" => return Error::InternalError(message),
            _ => {} // fall through
        }
    }
    let cause = convert_js_cause(err.cause(), depth);
    Error::UnknownJsError {
        original: err.into(),
        name,
        message,
        code,
        cause,
    }
}

/// Classify an arbitrary `JsValue` into an `Error`, threading depth so any
/// nested `js_sys::Error` causes are decoded with one less hop available.
fn from_js_value_with_depth(v: JsValue, depth: u32) -> Error {
    if let Ok(err) = v.clone().dyn_into::<js_sys::Error>() {
        return convert_js_error_with_depth(err, depth);
    }
    if let Some(message) = v.as_string() {
        return Error::JsError(message);
    }
    Error::Internal(v)
}

fn convert_js_cause(cause: JsValue, depth: u32) -> Option<Box<Error>> {
    if cause.is_null_or_undefined() || depth == 0 {
        return None;
    }
    Some(Box::new(from_js_value_with_depth(cause, depth - 1)))
}

impl From<js_sys::Error> for Error {
    fn from(err: js_sys::Error) -> Self {
        convert_js_error_with_depth(err, MAX_CAUSE_DEPTH)
    }
}

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
        Self::RustError(format!("Invalid header name: {value:?}"))
    }
}

#[cfg(feature = "http")]
impl From<http::header::InvalidHeaderValue> for Error {
    fn from(value: http::header::InvalidHeaderValue) -> Self {
        Self::RustError(format!("Invalid header value: {value:?}"))
    }
}

#[cfg(feature = "timezone")]
impl From<chrono_tz::ParseError> for Error {
    fn from(_value: chrono_tz::ParseError) -> Self {
        Self::RustError("Invalid timezone".to_string())
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

impl From<KvError> for Error {
    fn from(e: KvError) -> Self {
        Self::KvError(e)
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
            // Leaf errors: no inner data
            Error::BadEncoding => f.write_str("content-type mismatch"),
            Error::BodyUsed => f.write_str("body has already been read"),
            Error::Infallible => f.write_str("infallible"),
            Error::RouteNoDataError => f.write_str("route has no corresponding shared data"),
            #[cfg(feature = "timezone")]
            Error::TimezoneError => f.write_str("invalid timezone"),
            // String-carrying variants: no source, inline the data
            Error::JsError(s) => f.write_str(s),
            Error::RustError(s) => f.write_str(s),
            Error::BindingError(name) => write!(f, "no binding found for `{name}`"),
            Error::Json((msg, status)) => write!(f, "{msg} (status: {status})"),
            // Wrapped Rust errors: source() exposes the inner; here we just
            // categorize. Avoids duplication when consumers walk the chain.
            Error::Io(_) => f.write_str("I/O error"),
            Error::Utf8Error(_) => f.write_str("UTF-8 decoding error"),
            Error::SerdeJsonError(_) => f.write_str("JSON serialization error"),
            Error::SerdeWasmBindgenError(_) => f.write_str("wasm-bindgen serialization error"),
            Error::RouteInsertError(_) => f.write_str("failed to insert route"),
            #[cfg(feature = "http")]
            Error::Http(_) => f.write_str("HTTP error"),
            #[cfg(feature = "http")]
            Error::StatusCode(_) => f.write_str("invalid HTTP status code"),
            // Wrapped types we're not refactoring right now
            #[cfg(feature = "d1")]
            Error::D1(e) => write!(f, "D1: {e:#?}"),
            Error::KvError(KvError::JavaScript(s)) => write!(f, "js error: {s:?}"),
            Error::KvError(KvError::Serialization(s)) => {
                write!(f, "unable to serialize/deserialize: {s}")
            }
            Error::KvError(KvError::InvalidKvStore(s)) => write!(f, "invalid kv store: {s}"),
            // Opaque JS value
            Error::Internal(v) => write!(f, "unrecognized JavaScript value: {v:?}"),
            // Email/coded variants: `VariantName: <upstream message>`.
            // PascalCase variant name acts as a stable, greppable identifier;
            // the upstream message provides specific context.
            Error::EmailRecipientSuppressed(msg)
            | Error::EmailRecipientNotAllowed(msg)
            | Error::RateLimitExceeded(msg)
            | Error::DailyLimitExceeded(msg)
            | Error::InternalError(msg) => {
                let name: &'static str = self.into();
                write!(f, "{name}: {msg}")
            }
            // JS catch-all: Node-style `Name [code]: message`
            Error::UnknownJsError {
                name,
                message,
                code,
                ..
            } => {
                let prefix = name.as_deref().unwrap_or("Error");
                match code {
                    Some(c) => write!(f, "{prefix} [{c}]: {message}"),
                    None => write!(f, "{prefix}: {message}"),
                }
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            // Wrapped Rust errors
            Error::Io(e) => Some(e),
            Error::Utf8Error(e) => Some(e),
            Error::SerdeJsonError(e) => Some(e),
            Error::SerdeWasmBindgenError(e) => Some(e),
            Error::RouteInsertError(e) => Some(e),
            #[cfg(feature = "http")]
            Error::Http(e) => Some(e),
            #[cfg(feature = "http")]
            Error::StatusCode(e) => Some(e),
            // JS error chain
            Error::UnknownJsError { cause: Some(c), .. } => Some(c.as_ref()),
            // Everything else: leaf errors
            _ => None,
        }
    }
}

impl From<JsValue> for Error {
    fn from(v: JsValue) -> Self {
        // Top-level conversion: full depth budget for any nested cause chain.
        // Real Error objects flow through `convert_js_error_with_depth` for
        // coded-error decoding (email codes, cause extraction).
        // Bare strings become `JsError`. Other JS values become `Internal`.
        from_js_value_with_depth(v, MAX_CAUSE_DEPTH)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl Error {
    /// Build a `JsValue` representation of this error without consuming it.
    ///
    /// For variants holding a stored `JsValue` (`UnknownJsError`, `Internal`),
    /// this clones the original — cheap, since `JsValue::clone` is a refcount
    /// bump on the V8 anchor table. Other variants reconstruct a fresh
    /// `js_sys::Error` with the same code/message as the consuming
    /// `From<Error> for JsValue` impl.
    pub fn to_js_value(&self) -> JsValue {
        match self {
            // Stored originals: clone the underlying handle.
            Error::UnknownJsError { original, .. } => original.clone(),
            Error::Internal(v) => v.clone(),
            // Bare string — matches what JS originally threw.
            Error::JsError(s) => JsValue::from_str(s),
            // Coded variants: reconstruct a js_sys::Error with the canonical code.
            // Preserves the contract that JS callers can branch on err.code.
            Error::EmailRecipientSuppressed(msg) => {
                build_coded_error(msg, "E_RECIPIENT_SUPPRESSED")
            }
            Error::EmailRecipientNotAllowed(msg) => build_coded_error(msg, "RCPT_NOT_ALLOWED"),
            Error::RateLimitExceeded(msg) => build_coded_error(msg, "E_RATE_LIMIT_EXCEEDED"),
            Error::DailyLimitExceeded(msg) => build_coded_error(msg, "E_DAILY_LIMIT_EXCEEDED"),
            Error::InternalError(msg) => build_coded_error(msg, "E_INTERNAL_SERVER_ERROR"),
            // Everything else: build a plain js_sys::Error from the
            // chain-walked Display string, so source() info isn't lost.
            other => js_sys::Error::new(&format_with_chain(other)).into(),
        }
    }
}

impl From<Error> for JsValue {
    fn from(e: Error) -> Self {
        match e {
            // Move out of variants holding a JsValue — avoids the clone that
            // `to_js_value` would do. Functionally identical otherwise.
            Error::UnknownJsError { original, .. } => original,
            Error::Internal(v) => v,
            other => other.to_js_value(),
        }
    }
}

/// Walk an error's `source()` chain, formatting each cause on its own
/// indented line under a `Caused by:` header. Matches the convention used
/// by `anyhow` and `eyre`. Used when serializing back to a `JsValue` so
/// the full chain is preserved in the rendered string.
fn format_with_chain(e: &(dyn std::error::Error)) -> String {
    let mut s = e.to_string();
    let mut current = e.source();
    while let Some(src) = current {
        s.push_str("\nCaused by:\n    ");
        s.push_str(&src.to_string());
        current = src.source();
    }
    s
}

fn build_coded_error(message: &str, code: &str) -> JsValue {
    let err = js_sys::Error::new(message);
    let _ = Reflect::set(&err, &"code".into(), &JsValue::from_str(code));
    err.into()
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
