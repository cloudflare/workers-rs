use std::{collections::HashMap, convert::TryInto};

pub use builder::*;

use js_sys::{JsString, Reflect, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use worker_sys::{
    FixedLengthStream as EdgeFixedLengthStream, R2Bucket as EdgeR2Bucket, R2Checksums,
    R2MultipartUpload as EdgeR2MultipartUpload, R2Object as EdgeR2Object,
    R2ObjectBody as EdgeR2ObjectBody, R2Objects as EdgeR2Objects,
    R2UploadedPart as EdgeR2UploadedPart,
};

use crate::{
    env::EnvBinding, ByteStream, Date, Error, FixedLengthStream, Headers, ResponseBody, Result,
};

mod builder;

/// An instance of the R2 bucket binding.
#[derive(Clone)]
pub struct Bucket {
    inner: EdgeR2Bucket,
}

impl Bucket {
    /// Retrieves the [Object] for the given key containing only object metadata, if the key exists.
    pub async fn head(&self, key: impl Into<String>) -> Result<Option<Object>> {
        let head_promise = self.inner.head(key.into())?;
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
            checksum: None,
            checksum_algorithm: "md5".into(),
        }
    }

    /// Deletes the given value and metadata under the associated key. Once the delete succeeds,
    /// returns void.
    ///
    /// R2 deletes are strongly consistent. Once the Promise resolves, all subsequent read
    /// operations will no longer see this key value pair globally.
    pub async fn delete(&self, key: impl Into<String>) -> Result<()> {
        let delete_promise = self.inner.delete(key.into())?;
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

    /// Creates a multipart upload.
    ///
    /// Returns a [MultipartUpload] value representing the newly created multipart upload.
    /// Once the multipart upload has been created, the multipart upload can be immediately
    /// interacted with globally, either through the Workers API, or through the S3 API.
    pub fn create_multipart_upload(
        &self,
        key: impl Into<String>,
    ) -> CreateMultipartUploadOptionsBuilder {
        CreateMultipartUploadOptionsBuilder {
            edge_bucket: &self.inner,
            key: key.into(),
            http_metadata: None,
            custom_metadata: None,
        }
    }

    /// Returns an object representing a multipart upload with the given `key` and `uploadId`.
    ///
    /// The operation does not perform any checks to ensure the validity of the `uploadId`,
    /// nor does it verify the existence of a corresponding active multipart upload.
    /// This is done to minimize latency before being able to call subsequent operations on the returned object.
    pub fn resume_multipart_upload(
        &self,
        key: impl Into<String>,
        upload_id: impl Into<String>,
    ) -> Result<MultipartUpload> {
        Ok(MultipartUpload {
            inner: self
                .inner
                .resume_multipart_upload(key.into(), upload_id.into())?
                .into(),
        })
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
            ObjectInner::NoBody(inner) => inner.key().unwrap(),
            ObjectInner::Body(inner) => inner.key().unwrap(),
        }
    }

    pub fn version(&self) -> String {
        match &self.inner {
            ObjectInner::NoBody(inner) => inner.version().unwrap(),
            ObjectInner::Body(inner) => inner.version().unwrap(),
        }
    }

    pub fn size(&self) -> u32 {
        match &self.inner {
            ObjectInner::NoBody(inner) => inner.size().unwrap(),
            ObjectInner::Body(inner) => inner.size().unwrap(),
        }
    }

    pub fn etag(&self) -> String {
        match &self.inner {
            ObjectInner::NoBody(inner) => inner.etag().unwrap(),
            ObjectInner::Body(inner) => inner.etag().unwrap(),
        }
    }

    pub fn http_etag(&self) -> String {
        match &self.inner {
            ObjectInner::NoBody(inner) => inner.http_etag().unwrap(),
            ObjectInner::Body(inner) => inner.http_etag().unwrap(),
        }
    }

    pub fn uploaded(&self) -> Date {
        match &self.inner {
            ObjectInner::NoBody(inner) => inner.uploaded().unwrap(),
            ObjectInner::Body(inner) => inner.uploaded().unwrap(),
        }
        .into()
    }

    pub fn http_metadata(&self) -> HttpMetadata {
        match &self.inner {
            ObjectInner::NoBody(inner) => inner.http_metadata().unwrap(),
            ObjectInner::Body(inner) => inner.http_metadata().unwrap(),
        }
        .into()
    }

    pub fn checksum(&self) -> R2Checksums {
        match &self.inner {
            ObjectInner::NoBody(inner) => inner.checksums().unwrap(),
            ObjectInner::Body(inner) => inner.checksums().unwrap(),
        }
        .into()
    }

    pub fn custom_metadata(&self) -> Result<HashMap<String, String>> {
        let metadata = match &self.inner {
            ObjectInner::NoBody(inner) => inner.custom_metadata().unwrap(),
            ObjectInner::Body(inner) => inner.custom_metadata().unwrap(),
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
            ObjectInner::NoBody(inner) => inner.range().unwrap(),
            ObjectInner::Body(inner) => inner.range().unwrap(),
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
            ObjectInner::Body(inner) => Some(inner.body_used().unwrap()),
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
        if self.inner.body_used()? {
            return Err(Error::BodyUsed);
        }

        let stream = self.inner.body()?;
        let stream = wasm_streams::ReadableStream::from_raw(stream.unchecked_into());
        Ok(ByteStream {
            inner: stream.into_stream(),
        })
    }

    /// Returns a [ResponseBody] containing the data in the [Object].
    ///
    /// This function can be used to hand off the [Object] data to the workers runtime for streaming
    /// to the client in a [crate::Response]. This ensures that the worker does not consume CPU time
    /// while the streaming occurs, which can be significant if instead [ObjectBody::stream] is used.
    pub fn response_body(self) -> Result<ResponseBody> {
        if self.inner.body_used()? {
            return Err(Error::BodyUsed);
        }

        Ok(ResponseBody::Stream(self.inner.body()?))
    }

    pub async fn bytes(self) -> Result<Vec<u8>> {
        let js_buffer = JsFuture::from(self.inner.array_buffer()?).await?;
        let js_buffer = Uint8Array::new(&js_buffer);
        let mut bytes = vec![0; js_buffer.length() as usize];
        js_buffer.copy_to(&mut bytes);

        Ok(bytes)
    }

    pub async fn text(self) -> Result<String> {
        String::from_utf8(self.bytes().await?).map_err(|e| Error::RustError(e.to_string()))
    }
}

/// [UploadedPart] represents a part that has been uploaded.
/// [UploadedPart] objects are returned from [upload_part](MultipartUpload::upload_part) operations
/// and must be passed to the [complete](MultipartUpload::complete) operation.
pub struct UploadedPart {
    inner: EdgeR2UploadedPart,
}

impl UploadedPart {
    pub fn part_number(&self) -> u16 {
        self.inner.part_number().unwrap()
    }

    pub fn etag(&self) -> String {
        self.inner.etag().unwrap()
    }
}

pub struct MultipartUpload {
    inner: EdgeR2MultipartUpload,
}

impl MultipartUpload {
    /// Uploads a single part with the specified part number to this multipart upload.
    ///
    /// Returns an [UploadedPart] object containing the etag and part number.
    /// These [UploadedPart] objects are required when completing the multipart upload.
    ///
    /// Getting hold of a value of this type does not guarantee that there is an active
    /// underlying multipart upload corresponding to that object.
    ///
    /// A multipart upload can be completed or aborted at any time, either through the S3 API,
    /// or by a parallel invocation of your Worker.
    /// Therefore it is important to add the necessary error handling code around each operation
    /// on the [MultipartUpload] object in case the underlying multipart upload no longer exists.
    pub async fn upload_part(
        &self,
        part_number: u16,
        value: impl Into<Data>,
    ) -> Result<UploadedPart> {
        let uploaded_part =
            JsFuture::from(self.inner.upload_part(part_number, value.into().into())?).await?;
        Ok(UploadedPart {
            inner: uploaded_part.into(),
        })
    }

    /// Request the upload id.
    pub async fn upload_id(&self) -> String {
        self.inner.upload_id().unwrap()
    }

    /// Aborts the multipart upload.
    pub async fn abort(&self) -> Result<()> {
        JsFuture::from(self.inner.abort()?).await?;
        Ok(())
    }

    /// Completes the multipart upload with the given parts.
    /// When the future is ready, the object is immediately accessible globally by any subsequent read operation.
    pub async fn complete(
        self,
        uploaded_parts: impl IntoIterator<Item = UploadedPart>,
    ) -> Result<Object> {
        let object = JsFuture::from(
            self.inner.complete(
                uploaded_parts
                    .into_iter()
                    .map(|part| part.inner.into())
                    .collect(),
            )?,
        )
        .await?;
        Ok(Object {
            inner: ObjectInner::Body(object.into()),
        })
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
            .unwrap()
            .into_iter()
            .map(|raw| Object {
                inner: ObjectInner::NoBody(raw),
            })
            .collect()
    }

    /// If true, indicates there are more results to be retrieved for the current
    /// [list](Bucket::list) request.
    pub fn truncated(&self) -> bool {
        self.inner.truncated().unwrap()
    }

    /// A token that can be passed to future [list](Bucket::list) calls to resume listing from that
    /// point. Only present if truncated is true.
    pub fn cursor(&self) -> Option<String> {
        self.inner.cursor().unwrap()
    }

    /// If a delimiter has been specified, contains all prefixes between the specified prefix and
    /// the next occurrence of the delimiter.
    ///
    /// For example, if no prefix is provided and the delimiter is '/', `foo/bar/baz` would return
    /// `foo` as a delimited prefix. If `foo/` was passed as a prefix with the same structure and
    /// delimiter, `foo/bar` would be returned as a delimited prefix.
    pub fn delimited_prefixes(&self) -> Vec<String> {
        self.inner
            .delimited_prefixes()
            .unwrap()
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
    ReadableStream(web_sys::ReadableStream),
    Stream(FixedLengthStream),
    Text(String),
    Bytes(Vec<u8>),
    Empty,
}

impl From<web_sys::ReadableStream> for Data {
    fn from(stream: web_sys::ReadableStream) -> Self {
        Data::ReadableStream(stream)
    }
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
            Data::ReadableStream(stream) => stream.into(),
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
