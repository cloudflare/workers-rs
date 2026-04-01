use std::collections::HashMap;

use js_sys::{ArrayBuffer, Function, Map as JsMap, Object, Promise, Uint8Array};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::ReadableStream;

use crate::kv::{self, KvError, ListResponse};

/// Deserializes a possibly-null JsValue into `Option<T>`.
fn deserialize_nullable<T: DeserializeOwned>(value: JsValue) -> Result<Option<T>, KvError> {
    if value.is_null() || value.is_undefined() {
        Ok(None)
    } else {
        Ok(Some(
            serde_wasm_bindgen::from_value(value).map_err(JsValue::from)?,
        ))
    }
}

/// Extracts and deserializes metadata from a JS object with a "metadata" field.
fn extract_metadata<M: DeserializeOwned>(entry: &JsValue) -> Result<Option<M>, KvError> {
    let metadata = kv::get(entry, "metadata")?;
    deserialize_nullable(metadata)
}

/// Builds the options JsValue for get/getWithMetadata calls.
fn build_get_options(
    cache_ttl: Option<u64>,
    value_type: Option<GetValueType>,
) -> Result<JsValue, KvError> {
    let ser = Serializer::json_compatible();
    Ok(GetOptions {
        cache_ttl,
        value_type,
    }
    .serialize(&ser)
    .map_err(JsValue::from)?)
}

/// A builder to configure put requests.
#[derive(Debug, Clone)]
#[must_use = "PutOptionsBuilder does nothing until you 'execute' it"]
pub struct PutOptionsBuilder {
    pub(crate) this: Object,
    pub(crate) put_function: Function,
    pub(crate) name: JsValue,
    pub(crate) value: JsValue,
    pub(crate) expiration: Option<u64>,
    pub(crate) expiration_ttl: Option<u64>,
    pub(crate) metadata: Option<Value>,
}

#[derive(Serialize)]
struct PutOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    expiration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "expirationTtl")]
    expiration_ttl: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<Value>,
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
        let ser = Serializer::json_compatible();
        let options_object = PutOptions {
            expiration: self.expiration,
            expiration_ttl: self.expiration_ttl,
            metadata: self.metadata,
        }
        .serialize(&ser)
        .map_err(JsValue::from)?;

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
#[derive(Debug, Clone)]
#[must_use = "ListOptionsBuilder does nothing until you 'execute' it"]
pub struct ListOptionsBuilder {
    pub(crate) this: Object,
    pub(crate) list_function: Function,
    pub(crate) limit: Option<u64>,
    pub(crate) cursor: Option<String>,
    pub(crate) prefix: Option<String>,
}

#[derive(Serialize)]
struct ListOptions {
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
        let ser = Serializer::json_compatible();
        let options_object = ListOptions {
            limit: self.limit,
            cursor: self.cursor,
            prefix: self.prefix,
        }
        .serialize(&ser)
        .map_err(JsValue::from)?;

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
#[derive(Debug, Clone)]
#[must_use = "GetOptionsBuilder does nothing until you 'get' it"]
pub struct GetOptionsBuilder {
    pub(crate) this: Object,
    pub(crate) get_function: Function,
    pub(crate) get_with_meta_function: Function,
    pub(crate) name: JsValue,
    pub(crate) cache_ttl: Option<u64>,
    pub(crate) value_type: Option<GetValueType>,
}

#[derive(Serialize)]
struct GetOptions {
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
        let options_object = build_get_options(self.cache_ttl, self.value_type)?;

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
        deserialize_nullable(value)
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

    /// Gets the value as a readable stream.
    pub async fn stream(self) -> Result<Option<ReadableStream>, KvError> {
        let value = self.value_type(GetValueType::Stream).get().await?;
        match value.dyn_into::<ReadableStream>() {
            Ok(stream) => Ok(Some(stream)),
            Err(_) => Ok(None),
        }
    }

    async fn get_with_metadata<M>(&self) -> Result<(JsValue, Option<M>), KvError>
    where
        M: DeserializeOwned,
    {
        let options_object = build_get_options(self.cache_ttl, self.value_type)?;

        let promise: Promise = self
            .get_with_meta_function
            .call2(&self.this, &self.name, &options_object)?
            .into();

        let pair = JsFuture::from(promise).await?;
        let value = kv::get(&pair, "value")?;
        let metadata = extract_metadata(&pair)?;

        Ok((value, metadata))
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
        Ok((deserialize_nullable(value)?, metadata))
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

    /// Gets the value as a readable stream and it's associated metadata.
    pub async fn stream_with_metadata<M>(
        self,
    ) -> Result<(Option<ReadableStream>, Option<M>), KvError>
    where
        M: DeserializeOwned,
    {
        let (value, metadata) = self
            .value_type(GetValueType::Stream)
            .get_with_metadata()
            .await?;

        match value.dyn_into::<ReadableStream>() {
            Ok(stream) => Ok((Some(stream), metadata)),
            Err(_) => Ok((None, metadata)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Copy)]
#[serde(rename_all = "camelCase")]
pub(crate) enum GetValueType {
    Text,
    ArrayBuffer,
    Stream,
    Json,
}

/// A builder to configure bulk get requests.
#[derive(Debug)]
#[must_use = "GetBulkOptionsBuilder does nothing until you 'get' it"]
pub struct GetBulkOptionsBuilder {
    pub(crate) this: Object,
    pub(crate) get_function: Function,
    pub(crate) get_with_meta_function: Function,
    pub(crate) keys: JsValue,
    pub(crate) cache_ttl: Option<u64>,
    pub(crate) value_type: Option<GetValueType>,
}

impl GetBulkOptionsBuilder {
    /// The cache_ttl parameter must be an integer that is greater than or equal to 60. It defines
    /// the length of time in seconds that a KV result is cached in the edge location that it is
    /// accessed from.
    pub fn cache_ttl(mut self, cache_ttl: u64) -> Self {
        self.cache_ttl = Some(cache_ttl);
        self
    }

    fn value_type(mut self, value_type: GetValueType) -> Self {
        self.value_type = Some(value_type);
        self
    }

    async fn get(self) -> Result<JsMap, KvError> {
        let options_object = build_get_options(self.cache_ttl, self.value_type)?;

        let promise: Promise = self
            .get_function
            .call2(&self.this, &self.keys, &options_object)?
            .into();
        let result = JsFuture::from(promise).await?;
        Ok(JsMap::from(result))
    }

    /// Gets all values as strings.
    pub async fn text(self) -> Result<HashMap<String, Option<String>>, KvError> {
        let map = self.value_type(GetValueType::Text).get().await?;
        let mut result = HashMap::new();
        map.for_each(&mut |value, key| {
            if let Some(k) = key.as_string() {
                result.insert(k, value.as_string());
            }
        });
        Ok(result)
    }

    /// Tries to deserialize all values to the generic type.
    pub async fn json<T>(self) -> Result<HashMap<String, Option<T>>, KvError>
    where
        T: DeserializeOwned,
    {
        let map = self.value_type(GetValueType::Json).get().await?;
        let mut result = HashMap::new();
        let mut last_err: Option<KvError> = None;
        map.for_each(&mut |value, key| {
            if last_err.is_some() {
                return;
            }
            if let Some(k) = key.as_string() {
                match deserialize_nullable(value) {
                    Ok(v) => {
                        result.insert(k, v);
                    }
                    Err(e) => {
                        last_err = Some(e);
                    }
                }
            }
        });
        if let Some(err) = last_err {
            return Err(err);
        }
        Ok(result)
    }

    async fn get_with_metadata(self) -> Result<JsMap, KvError> {
        let options_object = build_get_options(self.cache_ttl, self.value_type)?;

        let promise: Promise = self
            .get_with_meta_function
            .call2(&self.this, &self.keys, &options_object)?
            .into();
        let result = JsFuture::from(promise).await?;
        Ok(JsMap::from(result))
    }

    /// Gets all values as strings along with their associated metadata.
    pub async fn text_with_metadata<M>(
        self,
    ) -> Result<HashMap<String, (Option<String>, Option<M>)>, KvError>
    where
        M: DeserializeOwned,
    {
        let map = self
            .value_type(GetValueType::Text)
            .get_with_metadata()
            .await?;
        let mut result = HashMap::new();
        let mut last_err: Option<KvError> = None;
        map.for_each(&mut |entry, key| {
            if last_err.is_some() {
                return;
            }
            if let Some(k) = key.as_string() {
                let value = match kv::get(&entry, "value") {
                    Ok(v) => v,
                    Err(e) => {
                        last_err = Some(KvError::from(e));
                        return;
                    }
                };
                match extract_metadata(&entry) {
                    Ok(metadata) => {
                        result.insert(k, (value.as_string(), metadata));
                    }
                    Err(e) => {
                        last_err = Some(e);
                    }
                }
            }
        });
        if let Some(err) = last_err {
            return Err(err);
        }
        Ok(result)
    }

    /// Tries to deserialize all values to the generic type along with their associated metadata.
    pub async fn json_with_metadata<T, M>(
        self,
    ) -> Result<HashMap<String, (Option<T>, Option<M>)>, KvError>
    where
        T: DeserializeOwned,
        M: DeserializeOwned,
    {
        let map = self
            .value_type(GetValueType::Json)
            .get_with_metadata()
            .await?;
        let mut result = HashMap::new();
        let mut last_err: Option<KvError> = None;
        map.for_each(&mut |entry, key| {
            if last_err.is_some() {
                return;
            }
            if let Some(k) = key.as_string() {
                let value_js = match kv::get(&entry, "value") {
                    Ok(v) => v,
                    Err(e) => {
                        last_err = Some(KvError::from(e));
                        return;
                    }
                };
                match (deserialize_nullable(value_js), extract_metadata(&entry)) {
                    (Ok(value), Ok(metadata)) => {
                        result.insert(k, (value, metadata));
                    }
                    (Err(e), _) | (_, Err(e)) => {
                        last_err = Some(e);
                    }
                }
            }
        });
        if let Some(err) = last_err {
            return Err(err);
        }
        Ok(result)
    }
}
