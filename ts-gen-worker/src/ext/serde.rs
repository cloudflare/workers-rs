use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::{Message, Request, Response, ResponseInitBuilder, Result};

/// JSON-compatible serialization: maps become objects, `u64` stays a number, missing
/// `Option`s become `null`, so `Response.json()` sees plain JSON data.
fn to_js<T: Serialize + ?Sized>(value: &T) -> Result<wasm_bindgen::JsValue> {
    Ok(value.serialize(&serde_wasm_bindgen::Serializer::json_compatible())?)
}

impl Request {
    /// Reads the body as JSON into `T`.
    pub async fn json_into<T: DeserializeOwned>(&self) -> Result<T> {
        let text = String::from(self.text().await?);
        Ok(serde_json::from_str(&text)?)
    }
}

impl Response {
    /// Reads the body as JSON into `T`.
    pub async fn json_into<T: DeserializeOwned>(&self) -> Result<T> {
        let text = String::from(self.text().await?);
        Ok(serde_json::from_str(&text)?)
    }
}

impl ResponseInitBuilder {
    /// A JSON response via `Response.json()`, which sets `content-type: application/json`.
    pub fn from_json<T: Serialize + ?Sized>(self, value: &T) -> Result<Response> {
        Ok(Response::json_with_init(&to_js(value)?, &self.build())?)
    }
}

impl Message {
    /// Deserializes the message body.
    pub fn body_into<T: DeserializeOwned>(&self) -> Result<T> {
        Ok(serde_wasm_bindgen::from_value(self.body())?)
    }
}
