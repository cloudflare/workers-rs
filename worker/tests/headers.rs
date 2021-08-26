#![cfg(target_arch = "wasm32")]
use std::assert_eq;

use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

use web_sys::{console, RequestInit};
use worker::prelude::*;

#[wasm_bindgen_test]
fn headers_set_append() {
    headers_set_append_helper().unwrap();
}

fn headers_set_append_helper() -> Result<()> {
    let mut headers = Headers::new();

    headers.set("Accept-Encoding", "deflate")?;
    assert_eq!(headers.has("Accept-Encoding")?, true);
    assert_eq!(headers.get("Accept-Encoding")?.as_deref(), Some("deflate"));

    headers.set("Accept-Encoding", "deflate")?;
    headers.set("Content-Type", "text/plain")?;
    assert_eq!(headers.get("Accept-Encoding")?.as_deref(), Some("deflate"));
    assert_eq!(headers.get("Content-Type")?.as_deref(), Some("text/plain"));

    headers.append("Accept-Encoding", "gzip")?;
    assert_eq!(
        headers.get("Accept-Encoding")?.as_deref(),
        Some("deflate, gzip")
    );
    Ok(())
}

#[wasm_bindgen_test]
fn request_headers() {
    request_headers_test().unwrap();
}

fn request_headers_test() -> Result<()> {
    let mut request = Request::new("https://google.com", "get")?;
    request
        .headers_mut()?
        .set("Content-Type", "application/json")?;
    assert_eq!(
        request.headers().get("Content-Type")?,
        Some("application/json".into())
    );

    let headers: Headers = [
        ("Content-Type", "text/plain"),
        ("Authorization", "Bearer hello"),
    ]
    .iter()
    .collect();

    let mut request = Request::new_with_init(
        "https://google.com",
        RequestInit::new()
            .headers(headers.as_ref())
            .method("POST")
            .body(Some(&"X".into())),
    )?;
    assert_eq!(
        request.headers().get("Authorization")?.as_deref(),
        Some("Bearer hello")
    );
    request
        .headers_mut()?
        .set("Authorization", "Bearer world")?;
    assert_eq!(
        request.headers().get("Authorization")?.as_deref(),
        Some("Bearer world")
    );

    // Simulate the request coming in from the 'fetch' eventListener
    let mut request = Request::from((
        "fetch".to_string(),
        edgeworker_sys::Request::new_with_str("helo")?,
    ));
    // Check that the header guard is immutable
    assert!(request.headers_mut().is_err());

    Ok(())
}

#[wasm_bindgen_test]
fn response_headers() {
    response_headers_test().unwrap();
}

fn response_headers_test() -> Result<()> {
    let mut response = Response::ok("Hello, World!")?;
    response
        .headers_mut()
        .set("Content-Type", "application/json")?;
    response.headers_mut().set("Content-Length", "450")?;

    assert_eq!(
        response.headers().get("Content-Type")?.as_deref(),
        Some("application/json")
    );
    assert_eq!(
        response.headers().get("Content-Length")?.as_deref(),
        Some("450")
    );

    let mut headers = Headers::new();
    headers.append("Accept-Encoding", "deflate")?;
    headers.append("Accept-Encoding", "gzip")?;

    *response.headers_mut() = headers;
    assert_eq!(response.headers().get("Content-Type")?, None);
    assert_eq!(
        response.headers().get("Accept-Encoding")?.as_deref(),
        Some("deflate, gzip")
    );

    Ok(())
}
