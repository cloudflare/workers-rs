use super::SomeSharedData;
use futures_util::stream::StreamExt;
use rand::Rng;
use std::time::Duration;
use worker::{console_log, Cache, Date, Delay, Env, Request, Response, ResponseBuilder, Result};

fn key(req: &Request) -> Result<Option<String>> {
    let uri = req.url()?;
    let mut segments = uri.path_segments().unwrap();
    Ok(segments.nth(2).map(|s| s.to_owned()))
}

#[worker::send]
pub async fn handle_cache_example(
    req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    console_log!("url: {}", req.url()?.to_string());
    let cache = Cache::default();
    let key = req.url()?.to_string();
    if let Some(resp) = cache.get(&key, true).await? {
        console_log!("Cache HIT!");
        Ok(resp)
    } else {
        console_log!("Cache MISS!");
        // Cache API respects Cache-Control headers. Setting s-max-age to 10
        // will limit the response to be in cache for 10 seconds max
        let mut resp = ResponseBuilder::new()
            .with_header("cache-control", "s-maxage=10")?
            .from_json(&serde_json::json!({ "timestamp": Date::now().as_millis() }))?;
        cache.put(key, resp.cloned()?).await?;
        Ok(resp)
    }
}

#[worker::send]
pub async fn handle_cache_api_get(
    req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    if let Some(key) = key(&req)? {
        let cache = Cache::default();
        if let Some(resp) = cache.get(format!("https://{key}"), true).await? {
            return Ok(resp);
        } else {
            return Response::ok("cache miss");
        }
    }
    Response::error("key missing", 400)
}

#[worker::send]
pub async fn handle_cache_api_put(
    req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    if let Some(key) = key(&req)? {
        let cache = Cache::default();
        // Cache API respects Cache-Control headers. Setting s-max-age to 10
        // will limit the response to be in cache for 10 seconds max
        let mut resp = ResponseBuilder::new()
            .with_header("cache-control", "s-maxage=10")?
            .from_json(&serde_json::json!({ "timestamp": Date::now().as_millis() }))?;
        cache.put(format!("https://{key}"), resp.cloned()?).await?;
        return Ok(resp);
    }
    Response::error("key missing", 400)
}

#[worker::send]
pub async fn handle_cache_api_delete(
    req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    if let Some(key) = key(&req)? {
        let cache = Cache::default();

        let res = cache.delete(format!("https://{key}"), true).await?;
        return Response::ok(serde_json::to_string(&res)?);
    }
    Response::error("key missing", 400)
}

#[worker::send]
pub async fn handle_cache_stream(
    req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    console_log!("url: {}", req.url()?.to_string());
    let cache = Cache::default();
    let key = req.url()?.to_string();
    if let Some(resp) = cache.get(&key, true).await? {
        console_log!("Cache HIT!");
        Ok(resp)
    } else {
        console_log!("Cache MISS!");
        let mut rng = rand::rng();
        let count = rng.random_range(0..10);
        let stream = futures_util::stream::repeat("Hello, world!\n")
            .take(count)
            .then(|text| async move {
                Delay::from(Duration::from_millis(50)).await;
                Result::Ok(text.as_bytes().to_vec())
            });

        // Cache API respects Cache-Control headers. Setting s-max-age to 10
        // will limit the response to be in cache for 10 seconds max
        let mut resp = ResponseBuilder::new()
            .with_header("cache-control", "s-maxage=10")?
            .from_stream(stream)?;
        console_log!("resp = {:?}", resp);
        cache.put(key, resp.cloned()?).await?;
        Ok(resp)
    }
}
