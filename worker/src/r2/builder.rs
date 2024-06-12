use std::{collections::HashMap, convert::TryFrom};

use js_sys::{Array, Date as JsDate, JsString, Object as JsObject, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use worker_sys::{
    R2Bucket as EdgeR2Bucket, R2HttpMetadata as R2HttpMetadataSys,
    R2MultipartUpload as EdgeR2MultipartUpload, R2Object as EdgeR2Object, R2Range as R2RangeSys,
};

use crate::{Date, Error, MultipartUpload, ObjectInner, Objects, Result};

use super::{Data, Object};

/// Options for configuring the [get](crate::r2::Bucket::get) operation.
pub struct GetOptionsBuilder<'bucket> {
    pub(crate) edge_bucket: &'bucket EdgeR2Bucket,
    pub(crate) key: String,
    pub(crate) only_if: Option<Conditional>,
    pub(crate) range: Option<Range>,
}

impl<'bucket> GetOptionsBuilder<'bucket> {
    /// Specifies that the object should only be returned given satisfaction of certain conditions
    /// in the [R2Conditional]. Refer to [Conditional operations](https://developers.cloudflare.com/r2/runtime-apis/#conditional-operations).
    pub fn only_if(mut self, only_if: Conditional) -> Self {
        self.only_if = Some(only_if);
        self
    }

    /// Specifies that only a specific length (from an optional offset) or suffix of bytes from the
    /// object should be returned. Refer to [Ranged reads](https://developers.cloudflare.com/r2/runtime-apis/#ranged-reads).
    pub fn range(mut self, range: Range) -> Self {
        self.range = Some(range);
        self
    }

    /// Executes the GET operation on the R2 bucket.
    pub async fn execute(self) -> Result<Option<Object>> {
        let name: String = self.key;
        let get_promise = self.edge_bucket.get(
            name,
            js_object! {
                "onlyIf" => self.only_if.map(JsObject::from),
                "range" => self.range.map(JsObject::from),
            }
            .into(),
        )?;

        let value = JsFuture::from(get_promise).await?;

        if value.is_null() {
            return Ok(None);
        }

        let res: EdgeR2Object = value.into();
        let inner = if JsString::from("bodyUsed").js_in(&res) {
            ObjectInner::Body(res.unchecked_into())
        } else {
            ObjectInner::NoBody(res)
        };

        Ok(Some(Object { inner }))
    }
}

/// You can pass an [Conditional] object to [GetOptionsBuilder]. If the condition check fails,
/// the body will not be returned. This will make [get](crate::r2::Bucket::get) have lower latency.
///
/// For more information about conditional requests, refer to [RFC 7232](https://datatracker.ietf.org/doc/html/rfc7232).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Conditional {
    /// Performs the operation if the object’s etag matches the given string.
    pub etag_matches: Option<String>,
    /// Performs the operation if the object’s etag does not match the given string.
    pub etag_does_not_match: Option<String>,
    /// Performs the operation if the object was uploaded before the given date.
    pub uploaded_before: Option<Date>,
    /// Performs the operation if the object was uploaded after the given date.
    pub uploaded_after: Option<Date>,
}

impl From<Conditional> for JsObject {
    fn from(val: Conditional) -> Self {
        js_object! {
            "etagMatches" => JsValue::from(val.etag_matches),
            "etagDoesNotMatch" => JsValue::from(val.etag_does_not_match),
            "uploadedBefore" => JsValue::from(val.uploaded_before.map(JsDate::from)),
            "uploadedAfter" => JsValue::from(val.uploaded_after.map(JsDate::from)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Range {
    /// Read `length` bytes starting at `offset`.
    OffsetWithLength { offset: u64, length: u64 },
    /// Read from `offset` to the end of the object.
    OffsetToEnd { offset: u64 },
    /// Read `length` bytes starting at the beginning of the object.
    Prefix { length: u64 },
    /// Read `suffix` bytes from the end of the object.
    Suffix { suffix: u64 },
}

const MAX_SAFE_INTEGER: u64 = js_sys::Number::MAX_SAFE_INTEGER as u64;

fn check_range_precision(value: u64) -> f64 {
    assert!(
        value <= MAX_SAFE_INTEGER,
        "Integer precision loss when converting to JavaScript number"
    );
    value as f64
}

impl From<Range> for JsObject {
    fn from(val: Range) -> Self {
        match val {
            Range::OffsetWithLength { offset, length } => js_object! {
                "offset" => Some(check_range_precision(offset)),
                "length" => Some(check_range_precision(length)),
                "suffix" => JsValue::UNDEFINED,
            },
            Range::OffsetToEnd { offset } => js_object! {
                "offset" => Some(check_range_precision(offset)),
                "length" => JsValue::UNDEFINED,
                "suffix" => JsValue::UNDEFINED,
            },
            Range::Prefix { length } => js_object! {
                "offset" => JsValue::UNDEFINED,
                "length" => Some(check_range_precision(length)),
                "suffix" => JsValue::UNDEFINED,
            },
            Range::Suffix { suffix } => js_object! {
                "offset" => JsValue::UNDEFINED,
                "length" => JsValue::UNDEFINED,
                "suffix" => Some(check_range_precision(suffix)),
            },
        }
    }
}

impl TryFrom<R2RangeSys> for Range {
    type Error = Error;

    fn try_from(val: R2RangeSys) -> Result<Self> {
        Ok(match (val.offset, val.length, val.suffix) {
            (Some(offset), Some(length), None) => Self::OffsetWithLength {
                offset: offset.round() as u64,
                length: length.round() as u64,
            },
            (Some(offset), None, None) => Self::OffsetToEnd {
                offset: offset.round() as u64,
            },
            (None, Some(length), None) => Self::Prefix {
                length: length.round() as u64,
            },
            (None, None, Some(suffix)) => Self::Suffix {
                suffix: suffix.round() as u64,
            },
            _ => return Err(Error::JsError("invalid range".into())),
        })
    }
}

/// Options for configuring the [put](crate::r2::Bucket::put) operation.
pub struct PutOptionsBuilder<'bucket> {
    pub(crate) edge_bucket: &'bucket EdgeR2Bucket,
    pub(crate) key: String,
    pub(crate) value: Data,
    pub(crate) http_metadata: Option<HttpMetadata>,
    pub(crate) custom_metadata: Option<HashMap<String, String>>,
    pub(crate) checksum: Option<Vec<u8>>,
    pub(crate) checksum_algorithm: String,
}

impl<'bucket> PutOptionsBuilder<'bucket> {
    /// Various HTTP headers associated with the object. Refer to [HttpMetadata].
    pub fn http_metadata(mut self, metadata: HttpMetadata) -> Self {
        self.http_metadata = Some(metadata);
        self
    }

    /// A map of custom, user-defined metadata that will be stored with the object.
    pub fn custom_metadata(mut self, metadata: impl Into<HashMap<String, String>>) -> Self {
        self.custom_metadata = Some(metadata.into());
        self
    }

    fn checksum_set(mut self, algorithm: &str, checksum: impl Into<Vec<u8>>) -> Self {
        self.checksum_algorithm = algorithm.into();
        self.checksum = Some(checksum.into());
        self
    }

    /// A md5 hash to use to check the received object’s integrity.
    pub fn md5(self, bytes: impl Into<Vec<u8>>) -> Self {
        self.checksum_set("md5", bytes)
    }

    /// A sha1 hash to use to check the received object’s integrity.
    pub fn sha1(self, bytes: impl Into<Vec<u8>>) -> Self {
        self.checksum_set("sha1", bytes)
    }

    /// A sha256 hash to use to check the received object’s integrity.
    pub fn sha256(self, bytes: impl Into<Vec<u8>>) -> Self {
        self.checksum_set("sha256", bytes)
    }

    /// A sha384 hash to use to check the received object’s integrity.
    pub fn sha384(self, bytes: impl Into<Vec<u8>>) -> Self {
        self.checksum_set("sha384", bytes)
    }

    /// A sha512 hash to use to check the received object’s integrity.
    pub fn sha512(self, bytes: impl Into<Vec<u8>>) -> Self {
        self.checksum_set("sha512", bytes)
    }

    /// Executes the PUT operation on the R2 bucket.
    pub async fn execute(self) -> Result<Object> {
        let value: JsValue = self.value.into();
        let name: String = self.key;

        let put_promise = self.edge_bucket.put(
            name,
            value,
            js_object! {
                "httpMetadata" => self.http_metadata.map(JsObject::from),
                "customMetadata" => match self.custom_metadata {
                    Some(metadata) => {
                        let obj = JsObject::new();
                        for (k, v) in metadata.into_iter() {
                            js_sys::Reflect::set(&obj, &JsString::from(k), &JsString::from(v))?;
                        }
                        obj.into()
                    }
                    None => JsValue::UNDEFINED,
                },
                self.checksum_algorithm => self.checksum.map(|bytes| {
                    let arr = Uint8Array::new_with_length(bytes.len() as _);
                    arr.copy_from(&bytes);
                    arr.buffer()
                }),
            }
            .into(),
        )?;
        let res: EdgeR2Object = JsFuture::from(put_promise).await?.into();
        let inner = if JsString::from("bodyUsed").js_in(&res) {
            ObjectInner::Body(res.unchecked_into())
        } else {
            ObjectInner::NoBody(res)
        };

        Ok(Object { inner })
    }
}

/// Options for configuring the [create_multipart_upload](crate::r2::Bucket::create_multipart_upload) operation.
pub struct CreateMultipartUploadOptionsBuilder<'bucket> {
    pub(crate) edge_bucket: &'bucket EdgeR2Bucket,
    pub(crate) key: String,
    pub(crate) http_metadata: Option<HttpMetadata>,
    pub(crate) custom_metadata: Option<HashMap<String, String>>,
}

impl<'bucket> CreateMultipartUploadOptionsBuilder<'bucket> {
    /// Various HTTP headers associated with the object. Refer to [HttpMetadata].
    pub fn http_metadata(mut self, metadata: HttpMetadata) -> Self {
        self.http_metadata = Some(metadata);
        self
    }

    /// A map of custom, user-defined metadata that will be stored with the object.
    pub fn custom_metadata(mut self, metadata: impl Into<HashMap<String, String>>) -> Self {
        self.custom_metadata = Some(metadata.into());
        self
    }

    /// Executes the multipart upload creation operation on the R2 bucket.
    pub async fn execute(self) -> Result<MultipartUpload> {
        let key: String = self.key;

        let create_multipart_upload_promise = self.edge_bucket.create_multipart_upload(
            key,
            js_object! {
                "httpMetadata" => self.http_metadata.map(JsObject::from),
                "customMetadata" => match self.custom_metadata {
                    Some(metadata) => {
                        let obj = JsObject::new();
                        for (k, v) in metadata.into_iter() {
                            js_sys::Reflect::set(&obj, &JsString::from(k), &JsString::from(v))?;
                        }
                        obj.into()
                    }
                    None => JsValue::UNDEFINED,
                },
            }
            .into(),
        )?;
        let inner: EdgeR2MultipartUpload = JsFuture::from(create_multipart_upload_promise)
            .await?
            .into();

        Ok(MultipartUpload { inner })
    }
}

/// Metadata that's automatically rendered into R2 HTTP API endpoints.
/// ```ignore
/// * contentType -> content-type
/// * contentLanguage -> content-language
/// etc...
/// ```
/// This data is echoed back on GET responses based on what was originally
/// assigned to the object (and can typically also be overriden when issuing
/// the GET request).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HttpMetadata {
    pub content_type: Option<String>,
    pub content_language: Option<String>,
    pub content_disposition: Option<String>,
    pub content_encoding: Option<String>,
    pub cache_control: Option<String>,
    pub cache_expiry: Option<Date>,
}

impl From<HttpMetadata> for JsObject {
    fn from(val: HttpMetadata) -> Self {
        js_object! {
            "contentType" => val.content_type,
            "contentLanguage" => val.content_language,
            "contentDisposition" => val.content_disposition,
            "contentEncoding" => val.content_encoding,
            "cacheControl" => val.cache_control,
            "cacheExpiry" => val.cache_expiry.map(JsDate::from),
        }
    }
}

impl From<R2HttpMetadataSys> for HttpMetadata {
    fn from(val: R2HttpMetadataSys) -> Self {
        Self {
            content_type: val.content_type().unwrap(),
            content_language: val.content_language().unwrap(),
            content_disposition: val.content_disposition().unwrap(),
            content_encoding: val.content_encoding().unwrap(),
            cache_control: val.cache_control().unwrap(),
            cache_expiry: val.cache_expiry().unwrap().map(Into::into),
        }
    }
}

/// Options for configuring the [list](crate::r2::Bucket::list) operation.
pub struct ListOptionsBuilder<'bucket> {
    pub(crate) edge_bucket: &'bucket EdgeR2Bucket,
    pub(crate) limit: Option<u32>,
    pub(crate) prefix: Option<String>,
    pub(crate) cursor: Option<String>,
    pub(crate) delimiter: Option<String>,
    pub(crate) include: Option<Vec<Include>>,
}

impl<'bucket> ListOptionsBuilder<'bucket> {
    /// The number of results to return. Defaults to 1000, with a maximum of 1000.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// The prefix to match keys against. Keys will only be returned if they start with given prefix.
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// An opaque token that indicates where to continue listing objects from. A cursor can be
    /// retrieved from a previous list operation.
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// The character to use when grouping keys.
    pub fn delimiter(mut self, delimiter: impl Into<String>) -> Self {
        self.delimiter = Some(delimiter.into());
        self
    }

    /// If you populate this array, then items returned will include this metadata.
    /// A tradeoff is that fewer results may be returned depending on how big this
    /// data is. For now the caps are TBD but expect the total memory usage for a list
    /// operation may need to be <1MB or even <128kb depending on how many list operations
    /// you are sending into one bucket. Make sure to look at `truncated` for the result
    /// rather than having logic like
    ///
    /// ```ignore
    /// # let limit = 10;
    /// while listed.len() < limit {
    ///     listed = bucket.list()
    ///         .limit(limit),
    ///         .include(vec![Include::CustomMetadata])
    ///         .execute()
    ///         .await?;
    /// }
    /// ```
    pub fn include(mut self, include: Vec<Include>) -> Self {
        self.include = Some(include);
        self
    }

    /// Executes the LIST operation on the R2 bucket.
    pub async fn execute(self) -> Result<Objects> {
        let list_promise = self.edge_bucket.list(
            js_object! {
                "limit" => self.limit,
                "prefix" => self.prefix,
                "cursor" => self.cursor,
                "delimiter" => self.delimiter,
                "include" => self
                    .include
                    .map(|include| {
                        let arr = Array::new();
                        for include in include {
                            arr.push(&JsString::from(match include {
                                Include::HttpMetadata => "httpMetadata",
                                Include::CustomMetadata => "customMetadata",
                            }));
                        }
                        arr.into()
                    })
                    .unwrap_or(JsValue::UNDEFINED),
            }
            .into(),
        )?;
        let inner = JsFuture::from(list_promise).await?.into();
        Ok(Objects { inner })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Include {
    HttpMetadata,
    CustomMetadata,
}

macro_rules! js_object {
    {$($key: expr => $value: expr),* $(,)?} => {{
        let obj = JsObject::new();
        $(
            {
                let res = ::js_sys::Reflect::set(&obj, &JsString::from($key), &JsValue::from($value));
                debug_assert!(res.is_ok(), "setting properties should never fail on our dictionary objects");
            }
        )*
        obj
    }};
}
pub(crate) use js_object;
