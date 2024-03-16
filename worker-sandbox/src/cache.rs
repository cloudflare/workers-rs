use super::SomeSharedData;
use futures_util::stream::StreamExt;
use rand::Rng;
use std::time::Duration;
use worker::{console_log, Cache, Date, Delay, Request, Response, Result, RouteContext};

pub async fn handle_cache_example(
    req: Request,
    _ctx: RouteContext<SomeSharedData>,
) -> Result<Response> {
    console_log!("url: {}", req.url()?.to_string());
    let cache = Cache::default();
    let key = req.url()?.to_string();
    if let Some(resp) = cache.get(&key, true).await? {
        console_log!("Cache HIT!");
        Ok(resp)
    } else {
        console_log!("Cache MISS!");
        let mut resp =
            Response::from_json(&serde_json::json!({ "timestamp": Date::now().as_millis() }))?;

        // Cache API respects Cache-Control headers. Setting s-max-age to 10
        // will limit the response to be in cache for 10 seconds max
        resp.headers_mut().set("cache-control", "s-maxage=10")?;
        cache.put(key, resp.cloned()?).await?;
        Ok(resp)
    }
}

pub async fn handle_cache_api_get(
    _req: Request,
    ctx: RouteContext<SomeSharedData>,
) -> Result<Response> {
    if let Some(key) = ctx.param("key") {
        let cache = Cache::default();
        if let Some(resp) = cache.get(format!("https://{key}"), true).await? {
            return Ok(resp);
        } else {
            return Response::ok("cache miss");
        }
    }
    Response::error("key missing", 400)
}

pub async fn handle_cache_api_put(
    _req: Request,
    ctx: RouteContext<SomeSharedData>,
) -> Result<Response> {
    if let Some(key) = ctx.param("key") {
        let cache = Cache::default();

        let mut resp =
            Response::from_json(&serde_json::json!({ "timestamp": Date::now().as_millis() }))?;

        // Cache API respects Cache-Control headers. Setting s-max-age to 10
        // will limit the response to be in cache for 10 seconds max
        resp.headers_mut().set("cache-control", "s-maxage=10")?;
        cache.put(format!("https://{key}"), resp.cloned()?).await?;
        return Ok(resp);
    }
    Response::error("key missing", 400)
}

pub async fn handle_cache_api_delete(
    _req: Request,
    ctx: RouteContext<SomeSharedData>,
) -> Result<Response> {
    if let Some(key) = ctx.param("key") {
        let cache = Cache::default();

        let res = cache.delete(format!("https://{key}"), true).await?;
        return Response::ok(serde_json::to_string(&res)?);
    }
    Response::error("key missing", 400)
}

pub async fn handle_cache_stream(
    req: Request,
    _ctx: RouteContext<SomeSharedData>,
) -> Result<Response> {
    console_log!("url: {}", req.url()?.to_string());
    let cache = Cache::default();
    let key = req.url()?.to_string();
    if let Some(resp) = cache.get(&key, true).await? {
        console_log!("Cache HIT!");
        Ok(resp)
    } else {
        console_log!("Cache MISS!");
        let mut rng = rand::thread_rng();
        let count = rng.gen_range(0..10);
        let stream = futures_util::stream::repeat("Hello, world!\n")
            .take(count)
            .then(|text| async move {
                Delay::from(Duration::from_millis(50)).await;
                Result::Ok(text.as_bytes().to_vec())
            });

        let mut resp = Response::from_stream(stream)?;
        console_log!("resp = {:?}", resp);
        // Cache API respects Cache-Control headers. Setting s-max-age to 10
        // will limit the response to be in cache for 10 seconds max
        resp.headers_mut().set("cache-control", "s-maxage=10")?;

        cache.put(key, resp.cloned()?).await?;
        Ok(resp)
    }
}
