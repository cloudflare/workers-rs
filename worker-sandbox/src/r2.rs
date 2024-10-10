use std::{collections::HashMap, sync::Mutex};

use futures_util::StreamExt;
use worker::{
    Bucket, Conditional, Data, Date, Env, FixedLengthStream, HttpMetadata, Include, Request,
    Response, Result,
};

use crate::SomeSharedData;

static SEEDED: Mutex<bool> = Mutex::new(false);

pub async fn seed_bucket(bucket: &Bucket) -> Result<()> {
    {
        let mut seeded = SEEDED.lock().unwrap();

        if *seeded {
            return Ok(());
        }

        *seeded = true;
    }

    bucket.put("no-props", "text".to_string()).execute().await?;
    bucket
        .put("no-props-no-body", Data::Empty)
        .execute()
        .await?;

    put_full_properties("with-props", bucket).await?;

    Ok(())
}

#[worker::send]
pub async fn list_empty(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let bucket = env.bucket("EMPTY_BUCKET")?;

    let objects = bucket.list().execute().await?;
    assert_eq!(objects.objects().len(), 0);
    assert!(!objects.truncated());
    assert_eq!(objects.cursor(), None);

    Response::ok("ok")
}

#[worker::send]
pub async fn list(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let bucket = env.bucket("SEEDED_BUCKET")?;
    seed_bucket(&bucket).await?;

    let objects = bucket.list().execute().await?;
    assert_eq!(objects.objects().len(), 3);
    assert!(!objects.truncated());
    assert_eq!(objects.cursor(), None);

    let objects = bucket.list().limit(1).execute().await?;
    assert_eq!(objects.objects().len(), 1);
    assert!(objects.truncated());
    let cursor = objects.cursor().unwrap();

    let objects_2 = bucket.list().cursor(cursor).execute().await?;
    assert_eq!(objects_2.objects().len(), 2);
    assert!(!objects_2.truncated());

    let with_prefix = bucket.list().prefix("no-").execute().await?;
    assert_eq!(with_prefix.objects().len(), 2);
    assert!(!with_prefix.truncated());

    let objects = bucket
        .list()
        .include(vec![Include::CustomMetadata])
        .execute()
        .await?;
    let count = objects
        .objects()
        .into_iter()
        .filter(|obj| {
            obj.custom_metadata()
                .ok()
                .map(|map| !map.is_empty())
                .unwrap_or(false)
        })
        .count();
    assert_eq!(count, 1);
    let objects = bucket
        .list()
        .include(vec![Include::HttpMetadata])
        .execute()
        .await?;
    let count = objects
        .objects()
        .into_iter()
        .filter(|obj| obj.http_metadata().content_type.is_some())
        .count();
    assert_eq!(count, 1);

    Response::ok("ok")
}

#[worker::send]
pub async fn get_empty(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let bucket = env.bucket("EMPTY_BUCKET")?;

    let object = bucket.get("doesnt-exist").execute().await?;
    assert!(object.is_none());

    // Ensure all properties are being properly read with no errors.
    let object = bucket
        .get("doesnt-exist-with-properties")
        .only_if(Conditional {
            etag_does_not_match: Some("a".into()),
            etag_matches: Some("b".into()),
            uploaded_after: Some(Date::now()),
            uploaded_before: Some(Date::now()),
        })
        .execute()
        .await?;
    assert!(object.is_none());

    Response::ok("ok")
}

#[worker::send]
pub async fn get(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let bucket = env.bucket("SEEDED_BUCKET")?;
    seed_bucket(&bucket).await?;

    let item = bucket.get("no-props").execute().await?.unwrap();
    let item_body = item.body().unwrap();

    assert_eq!(item_body.text().await?, "text");

    let (http_metadata, custom_metadata) = dummy_properties();
    let item = bucket.get("with-props").execute().await?.unwrap();
    let item_body = item.body().unwrap();
    assert_eq!(item_body.text().await?, "example");
    let uploaded_custom_metadata = item.custom_metadata()?;
    assert_eq!(uploaded_custom_metadata, custom_metadata);
    let uploaded_http_metadata = item.http_metadata();
    assert_eq!(uploaded_http_metadata, http_metadata);

    Response::ok("ok")
}

#[worker::send]
pub async fn put(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let bucket = env.bucket("PUT_BUCKET")?;

    // R2 requires that we use a fixed-length-stream for the body.
    let stream = futures_util::stream::repeat_with(|| Ok(vec![0u8; 16])).take(16);
    let fixed_stream = FixedLengthStream::wrap(stream, 16 * 16);

    bucket.put("text", "text".to_string()).execute().await?;
    bucket.put("bytes", vec![0u8; 32]).execute().await?;
    bucket.put("empty", Data::Empty).execute().await?;
    bucket.put("stream", fixed_stream).execute().await?;

    // Now let's get the objects again manually and make sure everything is in-tact.

    // Internally `.text()` calls `.bytes()` which calls `.stream()`, so most cases are covered
    // by just this check.
    let text_obj = bucket.get("text").execute().await?.unwrap();
    let text = text_obj.body().unwrap();
    assert_eq!(text.text().await?, "text");

    // Ensure that the empty object exists, but don't have a body.
    let empty_obj = bucket.get("empty").execute().await?.unwrap();

    // Miniflare behavior mismatch, in Miniflare an empty body will just return an object without
    // a body property. But in workerd it will only return an object without a body property in the
    // event that a condition failed
    if let Some(body) = empty_obj.body() {
        assert_eq!(body.bytes().await?.len(), 0)
    }

    Response::ok("ok")
}

#[worker::send]
pub async fn put_properties(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let bucket = env.bucket("PUT_BUCKET")?;
    let (http_metadata, custom_metadata, object_with_props) =
        put_full_properties("with_props", &bucket).await?;

    let uploaded_custom_metadata = object_with_props.custom_metadata()?;
    assert_eq!(uploaded_custom_metadata, custom_metadata);
    let uploaded_http_metadata = object_with_props.http_metadata();
    assert_eq!(uploaded_http_metadata, http_metadata);

    Response::ok("ok")
}

#[worker::send]
pub async fn put_multipart(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    const R2_MULTIPART_CHUNK_MIN_SIZE: usize = 5 * 1_024 * 1_024; // 5MiB.
                                                                  // const TEST_CHUNK_COUNT: usize = 3;

    let bucket = env.bucket("PUT_BUCKET")?;

    let upload = bucket
        .create_multipart_upload("multipart_upload")
        .execute()
        .await?;

    // R2 requires chunks – except for the last one – to be at least 5MiB long.
    let chunk_sizes = [
        R2_MULTIPART_CHUNK_MIN_SIZE + 100,
        R2_MULTIPART_CHUNK_MIN_SIZE + 200,
        500,
    ];
    let mut uploaded_parts = vec![];
    for (chunk_index, chunk_size) in chunk_sizes.iter().copied().enumerate() {
        let chunk = vec![chunk_index as u8; chunk_size];
        uploaded_parts.push(upload.upload_part(chunk_index as u16, chunk).await?);
    }
    upload.complete(uploaded_parts).await?;

    // Now let's get the object again and ensure it consists of all three parts that were uploaded.
    let complete_object = bucket.get("multipart_upload").execute().await?.unwrap();
    let complete_object_body = complete_object.body().unwrap();
    let complete_object_bytes = complete_object_body.bytes().await?;

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

    Response::ok("ok")
}

#[worker::send]
pub async fn delete(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let bucket = env.bucket("DELETE_BUCKET")?;

    bucket.put("key", Data::Empty).execute().await?;

    let objects = bucket.list().execute().await?;
    assert_eq!(objects.objects().len(), 1);

    bucket.delete("key").await?;

    let objects = bucket.list().execute().await?;
    assert_eq!(objects.objects().len(), 0);

    Response::ok("ok")
}

async fn put_full_properties(
    name: &str,
    bucket: &Bucket,
) -> Result<(HttpMetadata, HashMap<String, String>, worker::Object)> {
    let (http_metadata, custom_metadata) = dummy_properties();
    let md5_hash: [u8; 16] = md5::compute("example").into();
    let object_with_props = bucket
        .put(name, "example".to_string())
        .http_metadata(http_metadata.clone())
        .custom_metadata(custom_metadata.clone())
        .md5(md5_hash)
        .execute()
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
        cache_expiry: Some(Date::now()),
    };
    let custom_metadata = {
        let mut map = HashMap::new();
        map.insert("a".into(), "b".into());
        map
    };
    (http_metadata, custom_metadata)
}
