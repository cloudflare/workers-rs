use js_sys::{ArrayBuffer, Function, Object, Promise, Uint8Array};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

use crate::{KvError, ListResponse};

/// A builder to configure put requests.
#[derive(Debug, Clone, Serialize)]
#[must_use = "PutOptionsBuilder does nothing until you 'execute' it"]
pub struct PutOptionsBuilder {
    #[serde(skip)]
    pub(crate) this: Object,
    #[serde(skip)]
    pub(crate) put_function: Function,
    #[serde(skip)]
    pub(crate) name: JsValue,
    #[serde(skip)]
    pub(crate) value: JsValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) expiration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "expirationTtl")]
    pub(crate) expiration_ttl: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) metadata: Option<Value>,
}

impl PutOptionsBuilder {
    /// When (expressed as a [unix timestamp](https://en.wikipedia.org/wiki/Unix_time)) the key
    /// value pair will expire in the store.
    pub fn expiration(mut self, expiration: u64) -> Self {
        self.expiration = Some(expiration);
        self
    }
    /// How many seconds until the key value pair will expire.
    pub fn expiration_ttl(mut self, expiration_ttl: u64) -> Self {
        self.expiration_ttl = Some(expiration_ttl);
        self
    }
    /// Metadata to be stored with the key value pair.
    pub fn metadata<T: Serialize>(mut self, metadata: T) -> Result<Self, KvError> {
        self.metadata = Some(serde_json::to_value(metadata)?);
        Ok(self)
    }
    /// Puts the value in the kv store.
    pub async fn execute(self) -> Result<(), KvError> {
        let options_object = serde_wasm_bindgen::to_value(&self).map_err(JsValue::from)?;
        let promise: Promise = self
            .put_function
            .call3(&self.this, &self.name, &self.value, &options_object)?
            .into();
        JsFuture::from(promise)
            .await
            .map(|_| ())
            .map_err(KvError::from)
    }
}

/// A builder to configure list requests.
#[derive(Debug, Clone, Serialize)]
#[must_use = "ListOptionsBuilder does nothing until you 'execute' it"]
pub struct ListOptionsBuilder {
    #[serde(skip)]
    pub(crate) this: Object,
    #[serde(skip)]
    pub(crate) list_function: Function,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) prefix: Option<String>,
}

impl ListOptionsBuilder {
    /// The maximum number of keys returned. The default is 1000, which is the maximum. It is
    /// unlikely that you will want to change this default, but it is included for completeness.
    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }
    /// A string returned by a previous response used to paginate the keys in the store.
    pub fn cursor(mut self, cursor: String) -> Self {
        self.cursor = Some(cursor);
        self
    }
    /// A prefix that all keys must start with for them to be included in the response.
    pub fn prefix(mut self, prefix: String) -> Self {
        self.prefix = Some(prefix);
        self
    }
    /// Lists the key value pairs in the kv store.
    pub async fn execute(self) -> Result<ListResponse, KvError> {
        let options_object = serde_wasm_bindgen::to_value(&self).map_err(JsValue::from)?;
        let promise: Promise = self
            .list_function
            .call1(&self.this, &options_object)?
            .into();

        let value = JsFuture::from(promise).await?;
        let resp = serde_wasm_bindgen::from_value(value).map_err(JsValue::from)?;
        Ok(resp)
    }
}

/// A builder to configure get requests.
#[derive(Debug, Clone, Serialize)]
#[must_use = "GetOptionsBuilder does nothing until you 'get' it"]
pub struct GetOptionsBuilder {
    #[serde(skip)]
    pub(crate) this: Object,
    #[serde(skip)]
    pub(crate) get_function: Function,
    #[serde(skip)]
    pub(crate) get_with_meta_function: Function,
    #[serde(skip)]
    pub(crate) name: JsValue,
    #[serde(rename = "cacheTtl", skip_serializing_if = "Option::is_none")]
    pub(crate) cache_ttl: Option<u64>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub(crate) value_type: Option<GetValueType>,
}

impl GetOptionsBuilder {
    /// The cache_ttl parameter must be an integer that is greater than or equal to 60. It defines
    /// the length of time in seconds that a KV result is cached in the edge location that it is
    /// accessed from. This can be useful for reducing cold read latency on keys that are read
    /// relatively infrequently. It is especially useful if your data is write-once or
    /// write-rarely, but is not recommended if your data is updated often and you need to see
    /// updates shortly after they're written, because writes that happen from other edge locations
    /// won't be visible until the cached value expires.
    pub fn cache_ttl(mut self, cache_ttl: u64) -> Self {
        self.cache_ttl = Some(cache_ttl);
        self
    }

    fn value_type(mut self, value_type: GetValueType) -> Self {
        self.value_type = Some(value_type);
        self
    }

    async fn get(self) -> Result<JsValue, KvError> {
        let options_object = serde_wasm_bindgen::to_value(&self).map_err(JsValue::from)?;
        let promise: Promise = self
            .get_function
            .call2(&self.this, &self.name, &options_object)?
            .into();
        JsFuture::from(promise).await.map_err(KvError::from)
    }

    /// Gets the value as a string.
    pub async fn text(self) -> Result<Option<String>, KvError> {
        let value = self.value_type(GetValueType::Text).get().await?;
        Ok(value.as_string())
    }

    /// Tries to deserialize the inner text to the generic type.
    pub async fn json<T>(self) -> Result<Option<T>, KvError>
    where
        T: DeserializeOwned,
    {
        let value = self.value_type(GetValueType::Json).get().await?;
        Ok(if value.is_null() {
            None
        } else {
            Some(serde_wasm_bindgen::from_value(value).map_err(JsValue::from)?)
        })
    }

    /// Gets the value as a byte slice.
    pub async fn bytes(self) -> Result<Option<Vec<u8>>, KvError> {
        let v = self.value_type(GetValueType::ArrayBuffer).get().await?;
        if ArrayBuffer::instanceof(&v) {
            let buffer = ArrayBuffer::from(v);
            let buffer = Uint8Array::new(&buffer);
            Ok(Some(buffer.to_vec()))
        } else {
            Ok(None)
        }
    }

    async fn get_with_metadata<M>(&self) -> Result<(JsValue, Option<M>), KvError>
    where
        M: DeserializeOwned,
    {
        let options_object = serde_wasm_bindgen::to_value(&self).map_err(JsValue::from)?;
        let promise: Promise = self
            .get_with_meta_function
            .call2(&self.this, &self.name, &options_object)?
            .into();

        let pair = JsFuture::from(promise).await?;
        let metadata = crate::get(&pair, "metadata")?;
        let value = crate::get(&pair, "value")?;

        Ok((
            value,
            if metadata.is_null() {
                None
            } else {
                Some(serde_wasm_bindgen::from_value(metadata).map_err(JsValue::from)?)
            },
        ))
    }

    /// Gets the value as a string and it's associated metadata.
    pub async fn text_with_metadata<M>(self) -> Result<(Option<String>, Option<M>), KvError>
    where
        M: DeserializeOwned,
    {
        let (value, metadata) = self
            .value_type(GetValueType::Text)
            .get_with_metadata()
            .await?;
        Ok((value.as_string(), metadata))
    }

    /// Tries to deserialize the inner text to the generic type along with it's metadata.
    pub async fn json_with_metadata<T, M>(self) -> Result<(Option<T>, Option<M>), KvError>
    where
        T: DeserializeOwned,
        M: DeserializeOwned,
    {
        let (value, metadata) = self
            .value_type(GetValueType::Json)
            .get_with_metadata()
            .await?;
        Ok((
            if value.is_null() {
                None
            } else {
                Some(serde_wasm_bindgen::from_value(value).map_err(JsValue::from)?)
            },
            metadata,
        ))
    }

    /// Gets the value as a byte slice and it's associated metadata.
    pub async fn bytes_with_metadata<M>(self) -> Result<(Option<Vec<u8>>, Option<M>), KvError>
    where
        M: DeserializeOwned,
    {
        let (value, metadata) = self
            .value_type(GetValueType::ArrayBuffer)
            .get_with_metadata()
            .await?;

        if ArrayBuffer::instanceof(&value) {
            let buffer = ArrayBuffer::from(value);
            let buffer = Uint8Array::new(&buffer);
            Ok((Some(buffer.to_vec()), metadata))
        } else {
            Ok((None, metadata))
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum GetValueType {
    Text,
    ArrayBuffer,
    Json,
}
