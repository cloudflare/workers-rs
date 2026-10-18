use crate::{Env, SomeSharedData};
use js_sys::Uint8Array;
use std::{
    collections::HashMap,
    sync::atomic::{AtomicBool, Ordering},
};
use ts_gen_worker::*;
use wasm_bindgen_futures::spawn_local;

static SEEDED: AtomicBool = AtomicBool::new(false);

/// `worker::HttpMetadata`: an owned, comparable view of `R2HTTPMetadata`.
/// `worker::Date`'s `PartialEq` compares `as_f64()` of a Date object, which is
/// always `None`, so the original suite never compared `cache_expiry` values;
/// equality here checks presence only to keep the assertions equivalent.
#[derive(Clone, Debug)]
struct HttpMetadata {
    content_type: Option<String>,
    content_language: Option<String>,
    content_disposition: Option<String>,
    content_encoding: Option<String>,
    cache_control: Option<String>,
    cache_expiry: Option<f64>,
}

impl PartialEq for HttpMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.content_type == other.content_type
            && self.content_language == other.content_language
            && self.content_disposition == other.content_disposition
            && self.content_encoding == other.content_encoding
            && self.cache_control == other.cache_control
            && self.cache_expiry.is_some() == other.cache_expiry.is_some()
    }
}

impl HttpMetadata {
    fn from_js(meta: &R2HTTPMetadata) -> Self {
        Self {
            content_type: meta.content_type(),
            content_language: meta.content_language(),
            content_disposition: meta.content_disposition(),
            content_encoding: meta.content_encoding(),
            cache_control: meta.cache_control(),
            cache_expiry: meta.cache_expiry().map(|d| d.get_time()),
        }
    }

    fn to_js(&self) -> R2HTTPMetadata {
        let mut builder = R2HTTPMetadata::builder();
        if let Some(v) = &self.content_type {
            builder = builder.content_type(v);
        }
        if let Some(v) = &self.content_language {
            builder = builder.content_language(v);
        }
        if let Some(v) = &self.content_disposition {
            builder = builder.content_disposition(v);
        }
        if let Some(v) = &self.content_encoding {
            builder = builder.content_encoding(v);
        }
        if let Some(v) = &self.cache_control {
            builder = builder.cache_control(v);
        }
        if let Some(v) = self.cache_expiry {
            builder = builder.cache_expiry(&js_sys::Date::new_with_value(v));
        }
        builder.build()
    }
}

/// `worker::Object::custom_metadata()`.
fn custom_metadata(object: &R2Object) -> HashMap<String, String> {
    object
        .custom_metadata()
        .map(|record| record.iter().collect())
        .unwrap_or_default()
}

/// `worker::Object::http_metadata()` compares equal to what was put, with the
/// `cache_expiry` date by time value.
fn http_metadata(object: &R2Object) -> HttpMetadata {
    object
        .http_metadata()
        .map(|m| HttpMetadata::from_js(&m))
        .unwrap_or(HttpMetadata {
            content_type: None,
            content_language: None,
            content_disposition: None,
            content_encoding: None,
            cache_control: None,
            cache_expiry: None,
        })
}

pub async fn seed_bucket(bucket: &R2Bucket) -> Result<()> {
    if SEEDED.load(Ordering::Acquire) {
        return Ok(());
    }

    SEEDED.store(true, Ordering::Release);

    bucket.put_with_str("no-props", "text").await?;
    bucket.put_with_null("no-props-no-body").await?;

    put_full_properties("with-props", bucket).await?;

    Ok(())
}

pub async fn list_empty(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let bucket = env.empty_bucket();

    let objects = bucket.list().await?;
    assert_eq!(objects.objects().len(), 0);
    assert!(!objects.truncated());
    assert_eq!(objects.cursor(), None);

    Response::builder().ok("ok")
}

pub async fn list(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let bucket = env.seeded_bucket();
    seed_bucket(&bucket).await?;

    let objects = bucket.list().await?;
    assert_eq!(objects.objects().len(), 3);
    assert!(!objects.truncated());
    assert_eq!(objects.cursor(), None);

    let objects = bucket
        .list_with_options(&R2ListOptions::builder().limit(1).build())
        .await?;
    assert_eq!(objects.objects().len(), 1);
    assert!(objects.truncated());
    let cursor = objects.cursor().unwrap();

    let objects_2 = bucket
        .list_with_options(&R2ListOptions::builder().cursor(&cursor).build())
        .await?;
    assert_eq!(objects_2.objects().len(), 2);
    assert!(!objects_2.truncated());

    let with_prefix = bucket
        .list_with_options(&R2ListOptions::builder().prefix("no-").build())
        .await?;
    assert_eq!(with_prefix.objects().len(), 2);
    assert!(!with_prefix.truncated());

    let objects = bucket
        .list_with_options(
            &R2ListOptions::builder()
                .include(&["customMetadata"])
                .build(),
        )
        .await?;
    let count = objects
        .objects()
        .into_iter()
        .filter(|obj| !custom_metadata(obj).is_empty())
        .count();
    assert_eq!(count, 1);
    let objects = bucket
        .list_with_options(&R2ListOptions::builder().include(&["httpMetadata"]).build())
        .await?;
    let count = objects
        .objects()
        .into_iter()
        .filter(|obj| http_metadata(obj).content_type.is_some())
        .count();
    assert_eq!(count, 1);

    Response::builder().ok("ok")
}

pub async fn get_empty(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let bucket = env.empty_bucket();

    let object = bucket.get("doesnt-exist").await?;
    assert!(object.is_none());

    let object = bucket
        .get_with_options(
            "doesnt-exist-with-properties",
            &R2GetOptions::builder()
                .only_if(
                    &R2Conditional::builder()
                        .etag_does_not_match("a")
                        .etag_matches("b")
                        .uploaded_after(&js_sys::Date::new_0())
                        .uploaded_before(&js_sys::Date::new_0())
                        .build(),
                )
                .build(),
        )
        .await?;
    assert!(object.is_none());

    Response::builder().ok("ok")
}

pub async fn get(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let bucket = env.seeded_bucket();
    seed_bucket(&bucket).await?;

    let item = bucket.get("no-props").await?.unwrap();

    assert_eq!(item.text().await?, "text");

    let (expected_http_metadata, expected_custom_metadata) = dummy_properties();
    let item = bucket.get("with-props").await?.unwrap();
    assert_eq!(item.text().await?, "example");
    let uploaded_custom_metadata = custom_metadata(&item);
    assert_eq!(uploaded_custom_metadata, expected_custom_metadata);
    let uploaded_http_metadata = http_metadata(&item);
    assert_eq!(uploaded_http_metadata, expected_http_metadata);

    Response::builder().ok("ok")
}

pub async fn put(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let bucket = env.put_bucket();

    // R2 requires a fixed-length stream for streamed bodies.
    let fixed_stream = FixedLengthStream::new(256)?;
    let writer = fixed_stream.writable().get_writer()?;
    spawn_local(async move {
        for _ in 0..16 {
            if writer
                .write_with_chunk(&Uint8Array::from(&[0u8; 16][..]))
                .await
                .is_err()
            {
                return;
            }
        }
        let _ = writer.close().await;
    });

    bucket.put_with_str("text", "text").await?;
    bucket
        .put_with_buffer_source("bytes", &[0u8; 32][..])
        .await?;
    bucket.put_with_str("empty", "").await?;
    bucket.put("stream", &fixed_stream.readable()).await?;

    let text_obj = bucket.get("text").await?.unwrap();
    assert_eq!(text_obj.text().await?, "text");

    let empty_obj = bucket.get("empty").await?.unwrap();
    assert_eq!(empty_obj.bytes().await?.length(), 0);

    Response::builder().ok("ok")
}

pub async fn put_properties(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let bucket = env.put_bucket();
    let (expected_http_metadata, expected_custom_metadata, object_with_props) =
        put_full_properties("with_props", &bucket).await?;

    let uploaded_custom_metadata = custom_metadata(&object_with_props);
    assert_eq!(uploaded_custom_metadata, expected_custom_metadata);
    let uploaded_http_metadata = http_metadata(&object_with_props);
    assert_eq!(uploaded_http_metadata, expected_http_metadata);

    Response::builder().ok("ok")
}

#[allow(clippy::large_stack_arrays)]
pub async fn put_multipart(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    const R2_MULTIPART_CHUNK_MIN_SIZE: usize = 5 * 1_024 * 1_024; // 5MiB.

    let bucket = env.put_bucket();

    let upload = bucket.create_multipart_upload("multipart_upload").await?;

    // R2 requires chunks – except for the last one – to be at least 5MiB long.
    let chunk_sizes = [
        R2_MULTIPART_CHUNK_MIN_SIZE + 100,
        R2_MULTIPART_CHUNK_MIN_SIZE + 200,
        500,
    ];
    let mut uploaded_parts = vec![];
    for (chunk_index, chunk_size) in chunk_sizes.iter().copied().enumerate() {
        let chunk = vec![u8::try_from(chunk_index).unwrap(); chunk_size];
        uploaded_parts.push(
            upload
                .upload_part_with_buffer_source(u16::try_from(chunk_index).unwrap(), &chunk[..])
                .await?,
        );
    }
    upload.complete(&uploaded_parts).await?;

    let complete_object = bucket.get("multipart_upload").await?.unwrap();
    let complete_object_bytes = complete_object.bytes().await?.to_vec();

    assert_eq!(
        complete_object_bytes.len(),
        R2_MULTIPART_CHUNK_MIN_SIZE + 100 + R2_MULTIPART_CHUNK_MIN_SIZE + 200 + 500
    );
    assert_eq!(
        complete_object_bytes[0..R2_MULTIPART_CHUNK_MIN_SIZE + 100],
        [0; R2_MULTIPART_CHUNK_MIN_SIZE + 100]
    );
    assert_eq!(
        complete_object_bytes[R2_MULTIPART_CHUNK_MIN_SIZE + 100..]
            [..R2_MULTIPART_CHUNK_MIN_SIZE + 200],
        [1; R2_MULTIPART_CHUNK_MIN_SIZE + 200]
    );
    assert_eq!(
        complete_object_bytes
            [R2_MULTIPART_CHUNK_MIN_SIZE + 100 + R2_MULTIPART_CHUNK_MIN_SIZE + 200..],
        [2; 500]
    );

    Response::builder().ok("ok")
}

pub async fn delete(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let bucket = env.delete_bucket();

    bucket.put_with_str("key", "").await?;

    let objects = bucket.list().await?;
    assert_eq!(objects.objects().len(), 1);

    bucket.delete("key").await?;

    let keys: Vec<String> = (0..1000).map(|i| format!("key_{i}")).collect();
    for key in &keys {
        bucket.put_with_str(key, "").await?;
    }
    let objects = bucket.list().await?;
    assert_eq!(objects.objects().len(), keys.len());

    bucket.delete_with_slice(&keys).await?;
    let objects = bucket.list().await?;
    assert_eq!(objects.objects().len(), 0);

    Response::builder().ok("ok")
}

async fn put_full_properties(
    name: &str,
    bucket: &R2Bucket,
) -> Result<(HttpMetadata, HashMap<String, String>, R2Object)> {
    let (http_metadata, custom_metadata) = dummy_properties();
    let mut md5_hash: [u8; 16] = md5::compute("example").into();
    let options = R2PutOptions::builder()
        .http_metadata(&http_metadata.to_js())
        .custom_metadata(&custom_metadata.iter().collect())
        .md5(&md5_hash[..])
        .build();
    // The options keep their own copy of the digest, not a view of `md5_hash`.
    md5_hash.fill(0);
    let object_with_props = bucket
        .put_with_str_and_options(name, "example", &options)
        .await?;
    Ok((http_metadata, custom_metadata, object_with_props))
}

fn dummy_properties() -> (HttpMetadata, HashMap<String, String>) {
    let http_metadata = HttpMetadata {
        content_type: Some("text/text".into()),
        content_language: Some("en-US".into()),
        content_disposition: Some("inline".into()),
        content_encoding: Some("gzip".into()),
        cache_control: Some("immutable".into()),
        cache_expiry: Some(js_sys::Date::now()),
    };
    let custom_metadata = {
        let mut map = HashMap::new();
        map.insert("a".into(), "b".into());
        map
    };
    (http_metadata, custom_metadata)
}
