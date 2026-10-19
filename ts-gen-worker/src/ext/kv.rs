//! `worker::KvStore`'s fluent API beside the generated `KVNamespace` methods. The
//! generated `get`/`put`/`list`/`delete` keep their names and raw shapes; the builders
//! add the typed reads (`text`/`json`/`bytes`/`stream`, with or without metadata), the
//! put options and the bulk `Map` decoding.

use std::collections::HashMap;

use js_sys::{ArrayBuffer, JsNullable, Map};
use wasm_bindgen::convert::{FromWasmAbi, Upcast};
use wasm_bindgen::JsStringLike;
#[cfg(feature = "serde")]
use wasm_bindgen::JsValue;

#[cfg(feature = "serde")]
use crate::KVNamespaceGetWithMetadataResult;
use crate::{
    KVNamespace, KVNamespaceGetOptionsPartial, KVNamespaceListOptions, KVNamespaceListResult,
    KVNamespacePutOptions, ReadableStream, Result,
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
        self.put_value(key, KvPutValue::Bytes(ArrayBuffer::from(value)))
    }

    /// A write of the stream's chunks; any chunk type (a byte stream, `Body::body()`)
    /// is accepted.
    pub fn put_stream<R>(&self, key: &str, value: ReadableStream<R>) -> KvPutBuilder
    where
        ReadableStream<R>: Upcast<ReadableStream>,
    {
        self.put_value(key, KvPutValue::Stream(value.upcast_into()))
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

#[cfg(feature = "serde")]
type TextEntry = KVNamespaceGetWithMetadataResult<String, JsValue>;

fn get_options(cache_ttl: Option<u64>) -> KVNamespaceGetOptionsPartial<String> {
    let builder = KVNamespaceGetOptionsPartial::builder();
    match cache_ttl {
        Some(ttl) => builder.cache_ttl(ttl),
        None => builder,
    }
    .build()
}

#[cfg(feature = "serde")]
fn parse_json<T: serde::de::DeserializeOwned>(text: Option<&str>) -> Result<Option<T>> {
    Ok(text.map(serde_json::from_str).transpose()?)
}

#[cfg(feature = "serde")]
fn metadata<V, M: serde::de::DeserializeOwned>(
    entry: &KVNamespaceGetWithMetadataResult<V, JsValue>,
) -> Result<Option<M>> {
    Ok(entry
        .metadata()
        .map(serde_wasm_bindgen::from_value)
        .transpose()?)
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

    fn options(&self) -> KVNamespaceGetOptionsPartial<String> {
        get_options(self.cache_ttl)
    }

    pub async fn text(self) -> Result<Option<String>> {
        let options = self.options();
        self.kv.get_text_with_options(&self.key, &options).await
    }

    /// The value parsed as JSON into `T`.
    #[cfg(feature = "serde")]
    pub async fn json<T: serde::de::DeserializeOwned>(self) -> Result<Option<T>> {
        parse_json(self.text().await?.as_deref())
    }

    pub async fn bytes(self) -> Result<Option<Vec<u8>>> {
        let options = self.options();
        let buffer = self
            .kv
            .get_array_buffer_with_options(&self.key, &options)
            .await?;
        Ok(buffer.map(Vec::from))
    }

    pub async fn stream(self) -> Result<Option<ReadableStream>> {
        let options = self.options();
        self.kv.get_stream_with_options(&self.key, &options).await
    }

    #[cfg(feature = "serde")]
    pub async fn text_with_metadata<M: serde::de::DeserializeOwned>(
        self,
    ) -> Result<(Option<String>, Option<M>)> {
        let options = self.options();
        let entry = self
            .kv
            .get_with_metadata_text_with_options(&self.key, &options)
            .await?;
        Ok((entry.value(), metadata(&entry)?))
    }

    #[cfg(feature = "serde")]
    pub async fn json_with_metadata<
        T: serde::de::DeserializeOwned,
        M: serde::de::DeserializeOwned,
    >(
        self,
    ) -> Result<(Option<T>, Option<M>)> {
        let (text, meta) = self.text_with_metadata().await?;
        Ok((parse_json(text.as_deref())?, meta))
    }

    #[cfg(feature = "serde")]
    pub async fn bytes_with_metadata<M: serde::de::DeserializeOwned>(
        self,
    ) -> Result<(Option<Vec<u8>>, Option<M>)> {
        let options = self.options();
        let entry = self
            .kv
            .get_with_metadata_array_buffer_with_options(&self.key, &options)
            .await?;
        Ok((entry.value().map(Vec::from), metadata(&entry)?))
    }

    #[cfg(feature = "serde")]
    pub async fn stream_with_metadata<M: serde::de::DeserializeOwned>(
        self,
    ) -> Result<(Option<ReadableStream>, Option<M>)> {
        let options = self.options();
        let entry = self
            .kv
            .get_with_metadata_stream_with_options(&self.key, &options)
            .await?;
        Ok((entry.value(), metadata(&entry)?))
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

    async fn text_map(&self) -> Result<Map<String, JsNullable<String>>> {
        let options = get_options(self.cache_ttl);
        self.kv
            .get_text_with_slice_and_options(&self.keys, &options)
            .await
    }

    #[cfg(feature = "serde")]
    async fn text_map_with_metadata(&self) -> Result<Map<String, TextEntry>> {
        let options = get_options(self.cache_ttl);
        self.kv
            .get_with_metadata_text_with_slice_and_options(&self.keys, &options)
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
        Self::collect(self.text_map().await?, |v| Ok(v.into_option()))
    }

    /// Every requested key parsed as JSON into `T`, `None` where absent.
    #[cfg(feature = "serde")]
    pub async fn json<T: serde::de::DeserializeOwned>(self) -> Result<HashMap<String, Option<T>>> {
        Self::collect(self.text_map().await?, |v| {
            parse_json(v.into_option().as_deref())
        })
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
            KvPutValue::Text(text) => kv.put_with_options(&key, &text, &options).await?,
            KvPutValue::Bytes(buffer) => {
                kv.put_with_buffer_source_and_options(&key, &buffer, &options)
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
    pub fn cursor(self, cursor: impl JsStringLike) -> Self {
        self.options.set_cursor(cursor);
        self
    }

    pub fn prefix(self, prefix: impl JsStringLike) -> Self {
        self.options.set_prefix(prefix);
        self
    }

    pub async fn execute(self) -> Result<KVNamespaceListResult> {
        self.kv.list_with_options(&self.options).await
    }
}
