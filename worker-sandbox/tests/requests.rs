use std::time::{Duration, Instant};

use serde::Deserialize;
use util::*;
use worker::http::StatusCode;

mod util;

#[test]
fn request() {
    let _ = get("request", |r| r);
}

#[test]
fn empty() {
    let body = get("empty", |r| r).text().unwrap();
    assert_eq!(body, "");
}

#[test]
fn body() {
    let body = get("body", |r| r).text().unwrap();
    assert_eq!(body, "body");
}

#[test]
fn status_code() {
    let status_code = reqwest::blocking::get("http://127.0.0.1:8787/status-code")
        .unwrap()
        .status();
    assert_eq!(status_code, StatusCode::IM_A_TEAPOT);
}

#[test]
fn headers() {
    expect_wrangler();

    let response = post("headers", |r| r.header("A", "B"));
    let headers = response.headers();

    assert_eq!(headers.get("A").map(|v| v.to_str().unwrap()), Some("B"));
    assert_eq!(
        headers.get("Hello").map(|v| v.to_str().unwrap()),
        Some("World!")
    );
}

#[test]
fn echo() {
    const TEXT: &str = "echo this body back";
    let body = post("echo", |req| req.body(TEXT)).text().unwrap();
    assert_eq!(body, TEXT);
}

#[test]
#[ignore = "does not work on miniflare https://github.com/cloudflare/miniflare/issues/59"]
fn async_text_echo() {
    const TEXT: &str = "Example text!";
    let body = get("async-text-echo", |req| req.body(TEXT)).text().unwrap();
    assert_eq!(body, TEXT);
}

#[test]
fn fetch() {
    let body = get("fetch", |r| r).text().unwrap();
    assert_eq!(body, "received response with status code 200");
}

#[test]
fn fetch_cancelled() {
    let body = get("fetch-cancelled", |r| r).text().unwrap();
    assert!(body.starts_with("AbortError:"));
}

#[test]
fn secret() {
    let body = get("secret", |r| r).text().unwrap();
    assert_eq!(body, "secret!");
}

#[test]
fn wait() {
    let then = Instant::now();
    get("wait-1s", |r| r);
    assert!(then.elapsed() >= Duration::from_secs(1));
}

#[test]
fn init_called() {
    let body = get("init-called", |r| r).text().unwrap();
    assert_eq!(body, "true");
}

#[test]
fn cache() {
    // First time should result in cache miss
    let body = get("cache", |r| r).text().unwrap();
    assert_eq!(body, "cache miss");

    // Add key to cache
    let body: serde_json::Value = put("cache", |r| r).json().unwrap();
    let expected_ts = &body["timestamp"];

    // Should now be cache hit
    let body: serde_json::Value = get("cache", |r| r).json().unwrap();
    assert_eq!(&body["timestamp"], expected_ts);

    // Delete key from cache
    let body: serde_json::Value = delete("cache", |r| r).json().unwrap();
    assert_eq!(body, "Success");

    // Make sure key is now deleted
    let body = get("cache", |r| r).text().unwrap();
    assert_eq!(body, "cache miss");

    // Another delete should fail
    let body: serde_json::Value = delete("cache", |r| r).json().unwrap();
    assert_eq!(body, "ResponseNotFound");
}

#[test]
fn kv() {
    #[derive(Deserialize)]
    struct Keys {
        keys: Vec<serde_json::Value>,
    }
    let keys: Keys = get("kv", |r| r).json().unwrap();
    assert_eq!(keys.keys[0]["name"], "foo");
}

#[test]
fn durable() {
    let body = get("durable", |r| r).text().unwrap();
    assert!(body.starts_with("[durable_object]"));
}

#[test]
fn durable_alarm() {
    let body = get("durable/alarm", |r| r).text().unwrap();
    assert_eq!(body, "false");

    // Sleep for 200 milliseconds to make sure the alarm is triggered.
    std::thread::sleep(std::time::Duration::from_millis(200));

    let body = get("durable/alarm", |r| r).text().unwrap();
    assert_eq!(body, "true");
}

#[test]
fn service_binding() {
    let body: String = get("service-binding", |r| r).text().unwrap();
    assert_eq!(body, "hello world");
}

#[test]
fn r2_list_empty() {
    let body = get("r2/list-empty", |r| r).text().unwrap();
    assert_eq!(body, "ok");
}

#[test]
fn r2_list() {
    let body = get("r2/list", |r| r).text().unwrap();
    assert_eq!(body, "ok");
}

#[test]
fn r2_get_empty() {
    let body = get("r2/get-empty", |r| r).text().unwrap();
    assert_eq!(body, "ok");
}

#[test]
fn r2_get() {
    let body = get("r2/get", |r| r).text().unwrap();
    assert_eq!(body, "ok");
}

#[test]
fn r2_put() {
    let body = put("r2/put", |r| r).text().unwrap();
    assert_eq!(body, "ok");
}

#[test]
fn r2_put_with_properties() {
    let body = put("r2/put-properties", |r| r).text().unwrap();
    assert_eq!(body, "ok");
}

#[test]
fn r2_delete() {
    let body = delete("r2/delete", |r| r).text().unwrap();
    assert_eq!(body, "ok");
}
