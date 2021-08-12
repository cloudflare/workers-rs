use thiserror::Error;
use wasm_bindgen::JsValue;

#[derive(Debug, Error)]
pub enum WorkerError {
    #[error("0:?")]
    JsError(JsValue),
}

impl From<JsValue> for WorkerError {
    fn from(e: JsValue) -> Self {
        Self::JsError(e)
    }
}
