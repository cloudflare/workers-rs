//! Bindings to Cloudflare Worker's [KV](https://developers.cloudflare.com/workers/runtime-apis/kv)
//! to be used ***inside*** of a worker's context.
//!
//! # Example
//! ```ignore
//! let kv = KvStore::create("Example")?;
//!
//! // Insert a new entry into the kv.
//! kv.put("example_key", "example_value")?
//!     .metadata(vec![1, 2, 3, 4]) // Use some arbitrary serialiazable metadata
//!     .execute()
//!     .await?;
//!
//! // NOTE: kv changes can take a minute to become visible to other workers.
//! // Get that same metadata.
//! let (value, metadata) = kv.get("example_key").text_with_metadata::<Vec<usize>>().await?;
//! ```
#[forbid(missing_docs)]
mod builder;

pub use builder::*;

use js_sys::{global, Function, Object, Promise, Reflect, Uint8Array};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

/// A binding to a Cloudflare KvStore.
#[derive(Clone)]
pub struct KvStore {
    pub(crate) this: Object,
    pub(crate) get_function: Function,
    pub(crate) get_with_meta_function: Function,
    pub(crate) put_function: Function,
    pub(crate) list_function: Function,
    pub(crate) delete_function: Function,
}

// Allows for attachment to axum router, as Workers will never allow multithreading.
unsafe impl Send for KvStore {}
unsafe impl Sync for KvStore {}

impl KvStore {
    /// Creates a new [`KvStore`] with the binding specified in your `wrangler.toml`.
    pub fn create(binding: &str) -> Result<Self, KvError> {
        let this = get(&global(), binding)?;

        // Ensures that the kv store exists.
        if this.is_undefined() {
            Err(KvError::InvalidKvStore(binding.into()))
        } else {
            Ok(Self {
                get_function: get(&this, "get")?.into(),
                get_with_meta_function: get(&this, "getWithMetadata")?.into(),
                put_function: get(&this, "put")?.into(),
                list_function: get(&this, "list")?.into(),
                delete_function: get(&this, "delete")?.into(),
                this: this.into(),
            })
        }
    }

    /// Creates a new [`KvStore`] with the binding specified in your `wrangler.toml`, using an
    /// alternative `this` value for arbitrary binding contexts.
    pub fn from_this(this: &JsValue, binding: &str) -> Result<Self, KvError> {
        let this = get(this, binding)?;

        // Ensures that the kv store exists.
        if this.is_undefined() {
            Err(KvError::InvalidKvStore(binding.into()))
        } else {
            Ok(Self {
                get_function: get(&this, "get")?.into(),
                get_with_meta_function: get(&this, "getWithMetadata")?.into(),
                put_function: get(&this, "put")?.into(),
                list_function: get(&this, "list")?.into(),
                delete_function: get(&this, "delete")?.into(),
                this: this.into(),
            })
        }
    }

    /// Fetches the value from the kv store by name.
    pub fn get(&self, name: &str) -> GetOptionsBuilder {
        GetOptionsBuilder {
            this: self.this.clone(),
            get_function: self.get_function.clone(),
            get_with_meta_function: self.get_with_meta_function.clone(),
            name: JsValue::from(name),
            cache_ttl: None,
            value_type: None,
        }
    }

    /// Puts data into the kv store.
    pub fn put<T: ToRawKvValue>(&self, name: &str, value: T) -> Result<PutOptionsBuilder, KvError> {
        Ok(PutOptionsBuilder {
            this: self.this.clone(),
            put_function: self.put_function.clone(),
            name: JsValue::from(name),
            value: value.raw_kv_value()?,
            expiration: None,
            expiration_ttl: None,
            metadata: None,
        })
    }

    /// Puts the specified byte slice into the kv store.
    pub fn put_bytes(&self, name: &str, value: &[u8]) -> Result<PutOptionsBuilder, KvError> {
        let typed_array = Uint8Array::new_with_length(value.len() as u32);
        typed_array.copy_from(value);
        let value: JsValue = typed_array.buffer().into();
        Ok(PutOptionsBuilder {
            this: self.this.clone(),
            put_function: self.put_function.clone(),
            name: JsValue::from(name),
            value,
            expiration: None,
            expiration_ttl: None,
            metadata: None,
        })
    }

    /// Lists the keys in the kv store.
    pub fn list(&self) -> ListOptionsBuilder {
        ListOptionsBuilder {
            this: self.this.clone(),
            list_function: self.list_function.clone(),
            limit: None,
            cursor: None,
            prefix: None,
        }
    }

    /// Deletes a key in the kv store.
    pub async fn delete(&self, name: &str) -> Result<(), KvError> {
        let name = JsValue::from(name);
        let promise: Promise = self.delete_function.call1(&self.this, &name)?.into();
        JsFuture::from(promise).await?;
        Ok(())
    }
}

/// The response for listing the elements in a KV store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse {
    /// A slice of all of the keys in the KV store.
    pub keys: Vec<Key>,
    /// If there are more keys that can be fetched using the response's cursor.
    pub list_complete: bool,
    /// A string used for paginating responses.
    pub cursor: Option<String>,
}

/// The representation of a key in the KV store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Key {
    /// The name of the key.
    pub name: String,
    /// When (expressed as a [unix timestamp](https://en.wikipedia.org/wiki/Unix_time)) the key
    /// value pair will expire in the store.
    pub expiration: Option<u64>,
    /// All metadata associated with the key.
    pub metadata: Option<Value>,
}

/// A simple error type that can occur during kv operations.
#[derive(Debug, thiserror::Error)]
pub enum KvError {
    #[error("js error: {0:?}")]
    JavaScript(JsValue),
    #[error("unable to serialize/deserialize: {0}")]
    Serialization(serde_json::Error),
    #[error("invalid kv store: {0}")]
    InvalidKvStore(String),
}

impl From<KvError> for JsValue {
    fn from(val: KvError) -> Self {
        match val {
            KvError::JavaScript(value) => value,
            KvError::Serialization(e) => format!("KvError::Serialization: {e}").into(),
            KvError::InvalidKvStore(binding) => {
                format!("KvError::InvalidKvStore: {binding}").into()
            }
        }
    }
}

impl From<JsValue> for KvError {
    fn from(value: JsValue) -> Self {
        Self::JavaScript(value)
    }
}

impl From<serde_json::Error> for KvError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization(value)
    }
}

/// A trait for things that can be converted to [`wasm_bindgen::JsValue`] to be passed to the kv.
pub trait ToRawKvValue {
    fn raw_kv_value(&self) -> Result<JsValue, KvError>;
}

impl ToRawKvValue for str {
    fn raw_kv_value(&self) -> Result<JsValue, KvError> {
        Ok(JsValue::from(self))
    }
}

impl<T: Serialize> ToRawKvValue for T {
    fn raw_kv_value(&self) -> Result<JsValue, KvError> {
        let value = serde_wasm_bindgen::to_value(self).map_err(JsValue::from)?;

        if value.as_string().is_some() {
            Ok(value)
        } else if let Some(number) = value.as_f64() {
            Ok(JsValue::from(number.to_string()))
        } else if let Some(boolean) = value.as_bool() {
            Ok(JsValue::from(boolean.to_string()))
        } else {
            js_sys::JSON::stringify(&value)
                .map(JsValue::from)
                .map_err(Into::into)
        }
    }
}

fn get(target: &JsValue, name: &str) -> Result<JsValue, JsValue> {
    Reflect::get(target, &JsValue::from(name))
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for KvError {
    fn into_response(self) -> axum::response::Response<axum::body::Body> {
        axum::response::Response::builder()
            .status(500)
            .body("INTERNAL SERVER ERROR".into())
            .unwrap()
    }
}
