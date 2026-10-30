use super::SomeSharedData;
use crate::Env;
use std::collections::HashMap;
use ts_gen_worker::*;

async fn limit(rate_limiter: &RateLimit, key: &str) -> Result<bool> {
    Ok(rate_limiter
        .limit(&RateLimitOptions::new(key))
        .await?
        .success())
}

pub async fn handle_rate_limit_check(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let rate_limiter = env.test_rate_limiter();

    let success = limit(&rate_limiter, "test-key").await?;

    Response::builder().from_json(&serde_json::json!({
        "success": success,
    }))
}

pub async fn handle_rate_limit_with_key(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let uri = req.url_parsed()?;
    let segments = uri.path_segments().unwrap().collect::<Vec<_>>();
    let key = segments.get(2).unwrap_or(&"default-key");

    let rate_limiter = env.test_rate_limiter();
    let success = limit(&rate_limiter, key).await?;

    Response::builder().from_json(&serde_json::json!({
        "success": success,
        "key": key,
    }))
}

pub async fn handle_rate_limit_bulk_test(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let rate_limiter = env.test_rate_limiter();

    let mut results = Vec::new();
    for i in 0..15 {
        let key = format!("bulk-test-{}", i % 3);
        let success = limit(&rate_limiter, &key).await?;
        results.push(serde_json::json!({
            "index": i,
            "key": key,
            "success": success,
        }));
    }

    Response::builder().from_json(&serde_json::json!({
        "results": results,
    }))
}

pub async fn handle_rate_limit_reset(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let rate_limiter = env.test_rate_limiter();

    let key = format!("reset-test-{}", js_sys::Date::now());

    let mut outcomes = HashMap::new();
    for i in 0..12 {
        let success = limit(&rate_limiter, &key).await?;
        outcomes.insert(format!("request_{}", i + 1), success);
    }

    Response::builder().from_json(&outcomes)
}
