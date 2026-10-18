use serde::de::DeserializeOwned;
use serde::Serialize;

use wasm_bindgen::JsValue;

use crate::{DurableObjectStorage, Message, Request, Response, ResponseInitBuilder, Result};

/// JSON-compatible serialization: maps become objects, `u64` stays a number, missing
/// `Option`s become `null`, so `Response.json()` sees plain JSON data.
fn to_js<T: Serialize + ?Sized>(value: &T) -> Result<wasm_bindgen::JsValue> {
    Ok(value.serialize(&serde_wasm_bindgen::Serializer::json_compatible())?)
}

impl Request {
    /// Reads the body as JSON into `T`.
    pub async fn json_into<T: DeserializeOwned>(&self) -> Result<T> {
        Ok(serde_json::from_str(&self.text().await?)?)
    }
}

impl Response {
    /// Reads the body as JSON into `T`.
    pub async fn json_into<T: DeserializeOwned>(&self) -> Result<T> {
        Ok(serde_json::from_str(&self.text().await?)?)
    }
}

impl ResponseInitBuilder {
    /// A JSON response via `Response.json()`, which sets `content-type: application/json`.
    pub fn from_json<T: Serialize + ?Sized>(self, value: &T) -> Result<Response> {
        Ok(Response::json_with_response_init(
            &to_js(value)?,
            &self.build(),
        )?)
    }
}

impl Message {
    /// Deserializes the message body.
    pub fn body_into<T: DeserializeOwned>(&self) -> Result<T> {
        Ok(serde_wasm_bindgen::from_value(self.body())?)
    }
}

impl DurableObjectStorage {
    /// `storage.get(key)` deserialized; `None` when the key is absent. Values go through
    /// `serde_wasm_bindgen` both ways, so a `put_serialized` value round-trips.
    pub async fn get_into<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        match self.get::<JsValue>(key).await?.into_option() {
            None => Ok(None),
            Some(value) => Ok(Some(serde_wasm_bindgen::from_value(value)?)),
        }
    }

    /// `storage.put(key, value)` with `value` serialized through `serde_wasm_bindgen`;
    /// maps become `Map`s, which structured clone preserves.
    pub async fn put_serialized<T: Serialize + ?Sized>(&self, key: &str, value: &T) -> Result<()> {
        self.put(key, &serde_wasm_bindgen::to_value(value)?).await
    }
}
