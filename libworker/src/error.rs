use thiserror::Error;
use wasm_bindgen::JsValue;

/// Any errors that may be encountered while working with Workers
#[derive(Debug, Error)]
pub enum WorkerError {
    /// Something went wrong in Javascript
    #[error("{message}\n{js_error:?}")]
    JsError {
        /// The original diagnostic from javascript
        js_error: JsValue,
        /// An error message describing what went wrong
        message: String,
    },
}
