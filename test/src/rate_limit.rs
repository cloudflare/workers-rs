use super::SomeSharedData;
use std::collections::HashMap;
use worker::{js_sys, Env, Request, Response, Result};

#[worker::send]
pub async fn handle_rate_limit_check(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let rate_limiter = env.rate_limiter("TEST_RATE_LIMITER")?;

    // Use a fixed key for testing
    let outcome = rate_limiter.limit("test-key".to_string()).await?;

    Response::from_json(&serde_json::json!({
        "success": outcome.success,
    }))
}

#[worker::send]
pub async fn handle_rate_limit_with_key(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let uri = req.url()?;
    let segments = uri.path_segments().unwrap().collect::<Vec<_>>();
    let key = segments.get(2).unwrap_or(&"default-key");

    let rate_limiter = env.rate_limiter("TEST_RATE_LIMITER")?;
    let outcome = rate_limiter.limit(key.to_string()).await?;

    Response::from_json(&serde_json::json!({
        "success": outcome.success,
        "key": key,
    }))
}

#[worker::send]
pub async fn handle_rate_limit_bulk_test(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let rate_limiter = env.rate_limiter("TEST_RATE_LIMITER")?;

    // Test multiple requests to verify rate limiting behavior
    let mut results = Vec::new();
    for i in 0..15 {
        let key = format!("bulk-test-{}", i % 3); // Use 3 different keys
        let outcome = rate_limiter.limit(key.clone()).await?;
        results.push(serde_json::json!({
            "index": i,
            "key": key,
            "success": outcome.success,
        }));
    }

    Response::from_json(&serde_json::json!({
        "results": results,
    }))
}

#[worker::send]
pub async fn handle_rate_limit_reset(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let rate_limiter = env.rate_limiter("TEST_RATE_LIMITER")?;

    // Use a unique key to avoid interference with other tests
    let key = format!("reset-test-{}", js_sys::Date::now());

    // Make multiple requests with the same key
    let mut outcomes = HashMap::new();
    for i in 0..12 {
        let outcome = rate_limiter.limit(key.clone()).await?;
        outcomes.insert(format!("request_{}", i + 1), outcome.success);
    }

    Response::from_json(&outcomes)
}
