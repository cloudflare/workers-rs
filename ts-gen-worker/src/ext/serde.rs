use serde::de::DeserializeOwned;
use serde::Serialize;

use js_sys::Object;
use wasm_bindgen::JsCast;

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
        Ok(Response::json_with_init(&to_js(value)?, &self.build())?)
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
        let value = self.get(key).await?;
        Ok(value.map(serde_wasm_bindgen::from_value).transpose()?)
    }

    /// `storage.put(key, value)` with `value` serialized through `serde_wasm_bindgen`;
    /// maps become `Map`s, which structured clone preserves.
    pub async fn put_serialized<T: Serialize + ?Sized>(&self, key: &str, value: &T) -> Result<()> {
        self.put(key, &serde_wasm_bindgen::to_value(value)?).await
    }

    /// `storage.put(entries)`: each field of `entries` (a struct or a string-keyed map) is
    /// stored under its name, its value serialized as `put_serialized` would.
    pub async fn put_entries_serialized<T: Serialize + ?Sized>(&self, entries: &T) -> Result<()> {
        let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
        let entries: Object = entries.serialize(&serializer)?.unchecked_into();
        self.put_with_entries(&entries).await
    }
}
