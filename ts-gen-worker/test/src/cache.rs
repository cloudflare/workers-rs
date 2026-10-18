use super::SomeSharedData;
use crate::util::sleep;
use crate::Env;
use js_sys::Uint8Array;
use ts_gen_worker::*;
use wasm_bindgen_futures::spawn_local;

fn key(req: &Request) -> Result<Option<String>> {
    let uri = req.url_parsed()?;
    let mut segments = uri.path_segments().unwrap();
    Ok(segments.nth(2).map(ToOwned::to_owned))
}

fn default_cache() -> Cache {
    caches.with(CacheStorage::default)
}

fn timestamp_response() -> Result<Response> {
    let headers = Headers::new()?;
    headers.set("cache-control", "s-maxage=10")?;
    headers.set("content-type", "application/json")?;
    Ok(Response::new_with_str_and_init(
        serde_json::json!({ "timestamp": js_sys::Date::now() as u64 }).to_string(),
        &ResponseInit::builder().headers(&headers).build(),
    )?)
}

pub async fn handle_cache_example(
    req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    console_log!("url: {}", req.url());
    let cache = default_cache();
    let key = req.url();
    let options = CacheQueryOptions::builder().ignore_method(true).build();
    if let Some(resp) = cache.match_with_str_and_options(&key, &options).await? {
        console_log!("Cache HIT!");
        Ok(resp)
    } else {
        console_log!("Cache MISS!");
        let resp = timestamp_response()?;
        cache.put_with_str(&key, &resp.clone()?).await?;
        Ok(resp)
    }
}

pub async fn handle_cache_api_get(
    req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    if let Some(key) = key(&req)? {
        let cache = default_cache();
        let options = CacheQueryOptions::builder().ignore_method(true).build();
        if let Some(resp) = cache
            .match_with_str_and_options(format!("https://{key}"), &options)
            .await?
        {
            return Ok(resp);
        }
        return Response::builder().ok("cache miss");
    }
    Response::builder().error("key missing", 400)
}

pub async fn handle_cache_api_put(
    req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    if let Some(key) = key(&req)? {
        let cache = default_cache();
        let resp = timestamp_response()?;
        cache
            .put_with_str(format!("https://{key}"), &resp.clone()?)
            .await?;
        return Ok(resp);
    }
    Response::builder().error("key missing", 400)
}

pub async fn handle_cache_api_delete(
    req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    if let Some(key) = key(&req)? {
        let cache = default_cache();
        let options = CacheQueryOptions::builder().ignore_method(true).build();
        let deleted = cache
            .delete_with_str_and_options(format!("https://{key}"), &options)
            .await?;
        // `worker::CacheDeletionOutcome` serializes as these two strings.
        let outcome = if deleted {
            "Success"
        } else {
            "ResponseNotFound"
        };
        return Response::builder().ok(serde_json::to_string(outcome).unwrap());
    }
    Response::builder().error("key missing", 400)
}

pub async fn handle_cache_stream(
    req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    console_log!("url: {}", req.url());
    let cache = default_cache();
    let key = req.url();
    let options = CacheQueryOptions::builder().ignore_method(true).build();
    if let Some(resp) = cache.match_with_str_and_options(&key, &options).await? {
        console_log!("Cache HIT!");
        Ok(resp)
    } else {
        console_log!("Cache MISS!");
        let count = (js_sys::Math::random() * 10.0) as usize;
        let transform = TransformStream::new_typed()?;
        let writer = transform.writable().get_writer()?;
        spawn_local(async move {
            for _ in 0..count {
                sleep(50.0).await;
                let chunk = Uint8Array::from("Hello, world!\n".as_bytes());
                if writer.write_with_chunk(&chunk).await.is_err() {
                    return;
                }
            }
            let _ = writer.close().await;
        });

        let headers = Headers::new()?;
        headers.set("cache-control", "s-maxage=10")?;
        let resp = Response::new_with_readable_stream_and_init(
            &transform.readable(),
            &ResponseInit::builder().headers(&headers).build(),
        )?;
        console_log!("resp = {resp:?}");
        cache.put_with_str(&key, &resp.clone()?).await?;
        Ok(resp)
    }
}
