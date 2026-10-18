//! `worker::KvStore`'s fluent API beside the generated `KVNamespace` methods. The
//! generated `get`/`put`/`list`/`delete` keep their names and raw shapes; the builders
//! add the typed reads (`text`/`json`/`bytes`/`stream`, with or without metadata), the
//! put options and the bulk `Map` decoding.

use std::collections::HashMap;

use js_sys::{ArrayBuffer, JsNullable, Map, Promise, Uint8Array};
use wasm_bindgen::convert::FromWasmAbi;
use wasm_bindgen::{JsCast, JsValue};

use crate::{
    KVNamespace, KVNamespaceGetOptions, KVNamespaceGetWithMetadataResult, KVNamespaceListOptions,
    KVNamespaceListResult, KVNamespacePutOptions, ReadableStream, Result,
};

impl KVNamespace {
    /// A typed read of `key`: `.text()`, `.json::<T>()`, `.bytes()`, `.stream()` and their
    /// `_with_metadata` forms.
    pub fn get_builder(&self, key: &str) -> KvGetBuilder {
        KvGetBuilder {
            kv: self.clone(),
            key: key.to_owned(),
            cache_ttl: None,
        }
    }

    /// A typed bulk read of up to 100 keys.
    pub fn get_bulk(&self, keys: &[impl AsRef<str>]) -> KvGetBulkBuilder {
        KvGetBulkBuilder {
            kv: self.clone(),
            keys: keys.iter().map(|k| k.as_ref().to_owned()).collect(),
            cache_ttl: None,
        }
    }

    /// A write of `value` (a string, or anything `Serialize` rendered as its JSON text)
    /// with options; `.execute()` runs it.
    pub fn put_builder<T: ToRawKvValue>(&self, key: &str, value: T) -> Result<KvPutBuilder> {
        Ok(self.put_value(key, KvPutValue::Text(value.raw_kv_value()?)))
    }

    pub fn put_bytes(&self, key: &str, value: &[u8]) -> KvPutBuilder {
        self.put_value(key, KvPutValue::Bytes(Uint8Array::from(value).buffer()))
    }

    pub fn put_stream(&self, key: &str, value: ReadableStream) -> KvPutBuilder {
        self.put_value(key, KvPutValue::Stream(value))
    }

    fn put_value(&self, key: &str, value: KvPutValue) -> KvPutBuilder {
        KvPutBuilder {
            kv: self.clone(),
            key: key.to_owned(),
            value,
            options: KVNamespacePutOptions::new(),
        }
    }

    /// A key listing with `.limit()`, `.cursor()`, `.prefix()`; `.execute()` runs it.
    pub fn list_builder(&self) -> KvListBuilder {
        KvListBuilder {
            kv: self.clone(),
            options: KVNamespaceListOptions::new(),
        }
    }
}

/// Values a KV put accepts: strings as they are, other `Serialize` types as JSON text
/// (numbers and booleans as their plain text).
pub trait ToRawKvValue {
    fn raw_kv_value(&self) -> Result<String>;
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> ToRawKvValue for T {
    fn raw_kv_value(&self) -> Result<String> {
        let value = serde_json::to_value(self)?;
        Ok(match value {
            serde_json::Value::String(s) => s,
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            other => other.to_string(),
        })
    }
}

#[cfg(not(feature = "serde"))]
impl ToRawKvValue for &str {
    fn raw_kv_value(&self) -> Result<String> {
        Ok((*self).to_owned())
    }
}

#[cfg(not(feature = "serde"))]
impl ToRawKvValue for String {
    fn raw_kv_value(&self) -> Result<String> {
        Ok(self.clone())
    }
}

/// The `getWithMetadata` entry with its value type erased: the same call returns a
/// string, an `ArrayBuffer` or a stream depending on the requested `type`.
type Entry = KVNamespaceGetWithMetadataResult<JsValue, JsValue>;

#[cfg(feature = "serde")]
type TextEntry = KVNamespaceGetWithMetadataResult<String, JsValue>;

fn get_options(value_type: &str, cache_ttl: Option<u64>) -> KVNamespaceGetOptions<String> {
    let builder = KVNamespaceGetOptions::builder(&value_type.to_string());
    match cache_ttl {
        Some(ttl) => builder.cache_ttl(ttl),
        None => builder,
    }
    .build()
}

/// KV reports a missing key as `null`, which `Option<JsValue>` getters keep as `Some`.
fn present(value: Option<JsValue>) -> Option<JsValue> {
    value.filter(|v| !v.is_null())
}

fn as_text(value: Option<JsValue>) -> Option<String> {
    value.and_then(|v| v.as_string())
}

fn as_bytes(value: Option<JsValue>) -> Option<Vec<u8>> {
    value
        .and_then(|v| v.dyn_into::<ArrayBuffer>().ok())
        .map(|buffer| Uint8Array::new(&buffer).to_vec())
}

fn as_stream(value: Option<JsValue>) -> Option<ReadableStream> {
    value.map(JsCast::unchecked_into)
}

#[cfg(feature = "serde")]
fn parse_json<T: serde::de::DeserializeOwned>(text: Option<&str>) -> Result<Option<T>> {
    Ok(text.map(serde_json::from_str).transpose()?)
}

#[cfg(feature = "serde")]
fn as_json<T: serde::de::DeserializeOwned>(value: Option<JsValue>) -> Result<Option<T>> {
    parse_json(as_text(value).as_deref())
}

#[cfg(feature = "serde")]
fn metadata<V, M: serde::de::DeserializeOwned>(
    entry: &KVNamespaceGetWithMetadataResult<V, JsValue>,
) -> Result<Option<M>> {
    match present(entry.metadata()) {
        Some(value) => Ok(Some(serde_wasm_bindgen::from_value(value)?)),
        None => Ok(None),
    }
}

#[derive(Debug, Clone)]
#[must_use = "a KvGetBuilder does nothing until one of its read methods is awaited"]
pub struct KvGetBuilder {
    kv: KVNamespace,
    key: String,
    cache_ttl: Option<u64>,
}

impl KvGetBuilder {
    /// Seconds the value may be served from the edge cache; at least 60.
    pub fn cache_ttl(mut self, cache_ttl: u64) -> Self {
        self.cache_ttl = Some(cache_ttl);
        self
    }

    /// The awaited `get` lowers to `Option<String>`, which would coerce an `ArrayBuffer` or
    /// stream result through the string path, so the promise is awaited as a `JsValue`.
    async fn raw(&self, value_type: &str) -> Result<Option<JsValue>> {
        let options = get_options(value_type, self.cache_ttl);
        let promise: Promise<JsValue> = self
            .kv
            .get_with_key_str_and_options_kv_namespace_get_options_promise(&self.key, &options)?
            .unchecked_into();
        Ok(present(Some(promise.await?)))
    }

    #[cfg(feature = "serde")]
    async fn raw_with_metadata(&self, value_type: &str) -> Result<Entry> {
        let options = get_options(value_type, self.cache_ttl);
        let entry = self
            .kv
            .get_with_metadata_with_key_str_and_options_kv_namespace_get_options(
                &self.key, &options,
            )
            .await?;
        Ok(entry.unchecked_into())
    }

    pub async fn text(self) -> Result<Option<String>> {
        Ok(as_text(self.raw("text").await?))
    }

    /// The value parsed as JSON into `T`.
    #[cfg(feature = "serde")]
    pub async fn json<T: serde::de::DeserializeOwned>(self) -> Result<Option<T>> {
        as_json(self.raw("text").await?)
    }

    pub async fn bytes(self) -> Result<Option<Vec<u8>>> {
        Ok(as_bytes(self.raw("arrayBuffer").await?))
    }

    pub async fn stream(self) -> Result<Option<ReadableStream>> {
        Ok(as_stream(self.raw("stream").await?))
    }

    #[cfg(feature = "serde")]
    pub async fn text_with_metadata<M: serde::de::DeserializeOwned>(
        self,
    ) -> Result<(Option<String>, Option<M>)> {
        let entry = self.raw_with_metadata("text").await?;
        Ok((as_text(present(entry.value())), metadata(&entry)?))
    }

    #[cfg(feature = "serde")]
    pub async fn json_with_metadata<
        T: serde::de::DeserializeOwned,
        M: serde::de::DeserializeOwned,
    >(
        self,
    ) -> Result<(Option<T>, Option<M>)> {
        let entry = self.raw_with_metadata("text").await?;
        Ok((as_json(present(entry.value()))?, metadata(&entry)?))
    }

    #[cfg(feature = "serde")]
    pub async fn bytes_with_metadata<M: serde::de::DeserializeOwned>(
        self,
    ) -> Result<(Option<Vec<u8>>, Option<M>)> {
        let entry = self.raw_with_metadata("arrayBuffer").await?;
        Ok((as_bytes(present(entry.value())), metadata(&entry)?))
    }

    #[cfg(feature = "serde")]
    pub async fn stream_with_metadata<M: serde::de::DeserializeOwned>(
        self,
    ) -> Result<(Option<ReadableStream>, Option<M>)> {
        let entry = self.raw_with_metadata("stream").await?;
        Ok((as_stream(present(entry.value())), metadata(&entry)?))
    }
}

#[derive(Debug, Clone)]
#[must_use = "a KvGetBulkBuilder does nothing until one of its read methods is awaited"]
pub struct KvGetBulkBuilder {
    kv: KVNamespace,
    keys: Vec<String>,
    cache_ttl: Option<u64>,
}

impl KvGetBulkBuilder {
    /// Seconds the values may be served from the edge cache; at least 60.
    pub fn cache_ttl(mut self, cache_ttl: u64) -> Self {
        self.cache_ttl = Some(cache_ttl);
        self
    }

    async fn text_map(&self) -> Result<Map<String, Option<String>>> {
        let options = get_options("text", self.cache_ttl);
        self.kv
            .get_with_key_slice_and_options_kv_namespace_get_options(&self.keys, &options)
            .await
    }

    #[cfg(feature = "serde")]
    async fn text_map_with_metadata(&self) -> Result<Map<String, TextEntry>> {
        let options = get_options("text", self.cache_ttl);
        self.kv
            .get_with_metadata_with_key_slice_and_options_kv_namespace_get_options(
                &self.keys, &options,
            )
            .await
    }

    fn collect<V: FromWasmAbi, T>(
        map: Map<String, V>,
        mut decode: impl FnMut(V) -> Result<T>,
    ) -> Result<HashMap<String, T>> {
        map.into_iter()
            .map(|(key, value)| Ok((key, decode(value)?)))
            .collect()
    }

    /// Every requested key, `None` where absent.
    pub async fn text(self) -> Result<HashMap<String, Option<String>>> {
        Self::collect(self.text_map().await?, Ok)
    }

    /// Every requested key parsed as JSON into `T`, `None` where absent.
    #[cfg(feature = "serde")]
    pub async fn json<T: serde::de::DeserializeOwned>(self) -> Result<HashMap<String, Option<T>>> {
        Self::collect(self.text_map().await?, |v| parse_json(v.as_deref()))
    }

    #[cfg(feature = "serde")]
    pub async fn text_with_metadata<M: serde::de::DeserializeOwned>(
        self,
    ) -> Result<HashMap<String, (Option<String>, Option<M>)>> {
        Self::collect(self.text_map_with_metadata().await?, |entry| {
            Ok((entry.value(), metadata(&entry)?))
        })
    }

    #[cfg(feature = "serde")]
    pub async fn json_with_metadata<
        T: serde::de::DeserializeOwned,
        M: serde::de::DeserializeOwned,
    >(
        self,
    ) -> Result<HashMap<String, (Option<T>, Option<M>)>> {
        Self::collect(self.text_map_with_metadata().await?, |entry| {
            Ok((parse_json(entry.value().as_deref())?, metadata(&entry)?))
        })
    }
}

#[derive(Debug)]
enum KvPutValue {
    Text(String),
    Bytes(ArrayBuffer),
    Stream(ReadableStream),
}

#[derive(Debug)]
#[must_use = "a KvPutBuilder does nothing until `execute` is awaited"]
pub struct KvPutBuilder {
    kv: KVNamespace,
    key: String,
    value: KvPutValue,
    options: KVNamespacePutOptions,
}

impl KvPutBuilder {
    /// Unix timestamp (seconds) at which the key expires.
    pub fn expiration(self, expiration: u64) -> Self {
        self.options.set_expiration(expiration);
        self
    }

    /// Seconds until the key expires.
    pub fn expiration_ttl(self, expiration_ttl: u64) -> Self {
        self.options.set_expiration_ttl(expiration_ttl);
        self
    }

    /// Metadata stored beside the value, as JSON-compatible data.
    #[cfg(feature = "serde")]
    pub fn metadata<T: serde::Serialize>(self, metadata: T) -> Result<Self> {
        let value = metadata.serialize(&serde_wasm_bindgen::Serializer::json_compatible())?;
        self.options.set_metadata(&value);
        Ok(self)
    }

    pub async fn execute(self) -> Result<()> {
        let KvPutBuilder {
            kv,
            key,
            value,
            options,
        } = self;
        match value {
            KvPutValue::Text(text) => kv.put_with_str_and_options(&key, &text, &options).await?,
            KvPutValue::Bytes(buffer) => {
                kv.put_with_array_buffer_and_options(&key, &buffer, &options)
                    .await?
            }
            KvPutValue::Stream(stream) => {
                kv.put_with_readable_stream_and_options(&key, &stream, &options)
                    .await?
            }
        };
        Ok(())
    }
}

#[derive(Debug)]
#[must_use = "a KvListBuilder does nothing until `execute` is awaited"]
pub struct KvListBuilder {
    kv: KVNamespace,
    options: KVNamespaceListOptions,
}

impl KvListBuilder {
    /// Maximum keys returned; 1000 is the default and the maximum.
    pub fn limit(self, limit: u64) -> Self {
        self.options.set_limit(limit);
        self
    }

    /// Continues a previous listing from its `cursor`.
    pub fn cursor(self, cursor: impl AsRef<str>) -> Self {
        self.options
            .set_cursor(&JsNullable::wrap(cursor.as_ref().into()));
        self
    }

    pub fn prefix(self, prefix: impl AsRef<str>) -> Self {
        self.options
            .set_prefix(&JsNullable::wrap(prefix.as_ref().into()));
        self
    }

    pub async fn execute(self) -> Result<KVNamespaceListResult> {
        self.kv.list_with_options(&self.options).await
    }
}
