use std::{collections::HashMap, convert::TryInto};

pub use builder::*;

use futures_util::StreamExt;
use js_sys::{JsString, Reflect, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use worker_sys::{
    r2::{
        R2Bucket as EdgeR2Bucket, R2Object as EdgeR2Object, R2ObjectBody as EdgeR2ObjectBody,
        R2Objects as EdgeR2Objects,
    },
    FixedLengthStream as EdgeFixedLengthStream,
};

use crate::{env::EnvBinding, ByteStream, Date, Error, FixedLengthStream, Headers, Result};

mod builder;

/// An instance of the R2 bucket binding.
pub struct Bucket {
    inner: EdgeR2Bucket,
}

impl Bucket {
    /// Retrieves the [Object] for the given key containing only object metadata, if the key exists.
    pub async fn head(&self, key: impl Into<String>) -> Result<Option<Object>> {
        let head_promise = self.inner.head(key.into());
        let value = JsFuture::from(head_promise).await?;

        if value.is_null() {
            return Ok(None);
        }

        Ok(Some(Object {
            inner: ObjectInner::NoBody(value.into()),
        }))
    }

    /// Retrieves the [Object] for the given key containing object metadata and the object body if
    /// the key exists. In the event that a precondition specified in options fails, get() returns
    /// an [Object] with no body.
    pub fn get(&self, key: impl Into<String>) -> GetOptionsBuilder {
        GetOptionsBuilder {
            edge_bucket: &self.inner,
            key: key.into(),
            only_if: None,
            range: None,
        }
    }

    /// Stores the given `value` and metadata under the associated `key`. Once the write succeeds,
    /// returns an [Object] containing metadata about the stored Object.
    ///
    /// R2 writes are strongly consistent. Once the future resolves, all subsequent read operations
    /// will see this key value pair globally.
    pub fn put(&self, key: impl Into<String>, value: impl Into<Data>) -> PutOptionsBuilder {
        PutOptionsBuilder {
            edge_bucket: &self.inner,
            key: key.into(),
            value: value.into(),
            http_metadata: None,
            custom_metadata: None,
            md5: None,
        }
    }

    /// Deletes the given value and metadata under the associated key. Once the delete succeeds,
    /// returns void.
    ///
    /// R2 deletes are strongly consistent. Once the Promise resolves, all subsequent read
    /// operations will no longer see this key value pair globally.
    pub async fn delete(&self, key: impl Into<String>) -> Result<()> {
        let delete_promise = self.inner.delete(key.into());
        JsFuture::from(delete_promise).await?;
        Ok(())
    }

    /// Returns an [Objects] containing a list of [Objects]s contained within the bucket. By
    /// default, returns the first 1000 entries.
    pub fn list(&self) -> ListOptionsBuilder {
        ListOptionsBuilder {
            edge_bucket: &self.inner,
            limit: None,
            prefix: None,
            cursor: None,
            delimiter: None,
            include: None,
        }
    }
}

impl EnvBinding for Bucket {
    const TYPE_NAME: &'static str = "R2Bucket";
}

impl JsCast for Bucket {
    fn instanceof(val: &JsValue) -> bool {
        val.is_instance_of::<EdgeR2Bucket>()
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        Self { inner: val.into() }
    }

    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}

impl From<Bucket> for JsValue {
    fn from(bucket: Bucket) -> Self {
        JsValue::from(bucket.inner)
    }
}

impl AsRef<JsValue> for Bucket {
    fn as_ref(&self) -> &JsValue {
        &self.inner
    }
}

/// [Object] is created when you [put](Bucket::put) an object into a [Bucket]. [Object] represents
/// the metadata of an object based on the information provided by the uploader. Every object that
/// you [put](Bucket::put) into a [Bucket] will have an [Object] created.
pub struct Object {
    inner: ObjectInner,
}

impl Object {
    pub fn key(&self) -> String {
        match &self.inner {
            ObjectInner::NoBody(inner) => inner.key(),
            ObjectInner::Body(inner) => inner.key(),
        }
    }

    pub fn version(&self) -> String {
        match &self.inner {
            ObjectInner::NoBody(inner) => inner.version(),
            ObjectInner::Body(inner) => inner.version(),
        }
    }

    pub fn size(&self) -> u32 {
        match &self.inner {
            ObjectInner::NoBody(inner) => inner.size(),
            ObjectInner::Body(inner) => inner.size(),
        }
    }

    pub fn etag(&self) -> String {
        match &self.inner {
            ObjectInner::NoBody(inner) => inner.etag(),
            ObjectInner::Body(inner) => inner.etag(),
        }
    }

    pub fn http_etag(&self) -> String {
        match &self.inner {
            ObjectInner::NoBody(inner) => inner.http_etag(),
            ObjectInner::Body(inner) => inner.http_etag(),
        }
    }

    pub fn uploaded(&self) -> Date {
        match &self.inner {
            ObjectInner::NoBody(inner) => inner.uploaded(),
            ObjectInner::Body(inner) => inner.uploaded(),
        }
        .into()
    }

    pub fn http_metadata(&self) -> HttpMetadata {
        match &self.inner {
            ObjectInner::NoBody(inner) => inner.http_metadata(),
            ObjectInner::Body(inner) => inner.http_metadata(),
        }
        .into()
    }

    pub fn custom_metadata(&self) -> Result<HashMap<String, String>> {
        let metadata = match &self.inner {
            ObjectInner::NoBody(inner) => inner.custom_metadata(),
            ObjectInner::Body(inner) => inner.custom_metadata(),
        };

        let keys = js_sys::Object::keys(&metadata).to_vec();
        let mut map = HashMap::with_capacity(keys.len());

        for key in keys {
            let key = key.unchecked_into::<JsString>();
            let value = Reflect::get(&metadata, &key)?.dyn_into::<JsString>()?;
            map.insert(key.into(), value.into());
        }

        Ok(map)
    }

    pub fn range(&self) -> Result<Range> {
        match &self.inner {
            ObjectInner::NoBody(inner) => inner.range(),
            ObjectInner::Body(inner) => inner.range(),
        }
        .try_into()
    }

    pub fn body(&self) -> Option<ObjectBody> {
        match &self.inner {
            ObjectInner::NoBody(_) => None,
            ObjectInner::Body(body) => Some(ObjectBody { inner: body }),
        }
    }

    pub fn body_used(&self) -> Option<bool> {
        match &self.inner {
            ObjectInner::NoBody(_) => None,
            ObjectInner::Body(inner) => Some(inner.body_used()),
        }
    }

    pub fn write_http_metadata(&self, headers: Headers) -> Result<()> {
        match &self.inner {
            ObjectInner::NoBody(inner) => inner.write_http_metadata(headers.0)?,
            ObjectInner::Body(inner) => inner.write_http_metadata(headers.0)?,
        };

        Ok(())
    }
}

/// The data contained within an [Object].
pub struct ObjectBody<'body> {
    inner: &'body EdgeR2ObjectBody,
}

impl<'body> ObjectBody<'body> {
    /// Reads the data in the [Object] via a [ByteStream].
    pub fn stream(self) -> Result<ByteStream> {
        if self.inner.body_used() {
            return Err(Error::BodyUsed);
        }

        let stream = self.inner.body();
        let stream = wasm_streams::ReadableStream::from_raw(stream.unchecked_into());
        Ok(ByteStream {
            inner: stream.into_stream(),
        })
    }

    pub async fn bytes(self) -> Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(self.inner.size() as usize);
        let mut stream = self.stream()?;

        while let Some(chunk) = stream.next().await {
            let mut chunk = chunk?;
            bytes.append(&mut chunk);
        }

        Ok(bytes)
    }

    pub async fn text(self) -> Result<String> {
        String::from_utf8(self.bytes().await?).map_err(|e| Error::RustError(e.to_string()))
    }
}

/// A series of [Object]s returned by [list](Bucket::list).
pub struct Objects {
    inner: EdgeR2Objects,
}

impl Objects {
    /// An [Vec] of [Object] matching the [list](Bucket::list) request.
    pub fn objects(&self) -> Vec<Object> {
        self.inner
            .objects()
            .into_iter()
            .map(|raw| Object {
                inner: ObjectInner::NoBody(raw),
            })
            .collect()
    }

    /// If true, indicates there are more results to be retrieved for the current
    /// [list](Bucket::list) request.
    pub fn truncated(&self) -> bool {
        self.inner.truncated()
    }

    /// A token that can be passed to future [list](Bucket::list) calls to resume listing from that
    /// point. Only present if truncated is true.
    pub fn cursor(&self) -> Option<String> {
        self.inner.cursor()
    }

    /// If a delimiter has been specified, contains all prefixes between the specified prefix and
    /// the next occurence of the delimiter.
    ///
    /// For example, if no prefix is provided and the delimiter is '/', `foo/bar/baz` would return
    /// `foo` as a delimited prefix. If `foo/` was passed as a prefix with the same structure and
    /// delimiter, `foo/bar` would be returned as a delimited prefix.
    pub fn delimited_prefixes(&self) -> Vec<String> {
        self.inner
            .delimited_prefixes()
            .into_iter()
            .map(Into::into)
            .collect()
    }
}

#[derive(Clone)]
pub(crate) enum ObjectInner {
    NoBody(EdgeR2Object),
    Body(EdgeR2ObjectBody),
}

pub enum Data {
    Stream(FixedLengthStream),
    Text(String),
    Bytes(Vec<u8>),
    Empty,
}

impl From<FixedLengthStream> for Data {
    fn from(stream: FixedLengthStream) -> Self {
        Data::Stream(stream)
    }
}

impl From<String> for Data {
    fn from(value: String) -> Self {
        Data::Text(value)
    }
}

impl From<Vec<u8>> for Data {
    fn from(value: Vec<u8>) -> Self {
        Data::Bytes(value)
    }
}

impl From<Data> for JsValue {
    fn from(data: Data) -> Self {
        match data {
            Data::Stream(stream) => {
                let stream_sys: EdgeFixedLengthStream = stream.into();
                stream_sys.readable().into()
            }
            Data::Text(text) => JsString::from(text).into(),
            Data::Bytes(bytes) => {
                let arr = Uint8Array::new_with_length(bytes.len() as u32);
                arr.copy_from(&bytes);
                arr.into()
            }
            Data::Empty => JsValue::NULL,
        }
    }
}
