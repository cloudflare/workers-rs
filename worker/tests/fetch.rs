#![cfg(target_arch = "wasm32")]
use std::{assert, assert_eq, assert_ne};

use serde_json::Value;
use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

use web_sys::console;
use worker::prelude::*;

#[wasm_bindgen_test]
async fn fetch_errors() {
    let response = Fetch::Url("ftp://example.com").fetch().await;
    assert!(response.is_err());

    let response = Fetch::Url("https://notarealwebsite.asdf").fetch().await;
    assert!(response.is_err());

    let response = Fetch::Url("https://username:password@example.com")
        .fetch()
        .await;
    assert!(response.is_err());
}

#[wasm_bindgen_test]
async fn using_fetch() {
    using_fetch_test().await.unwrap()
}

async fn using_fetch_test() -> Result<()> {
    let response = Fetch::Url("https://reqres.in/api/users").fetch().await;
    assert!(response.is_ok());

    let request = Request::new("https://reqres.in/api/users", "POST")?;
    let mut response = Fetch::Request(&request).fetch().await;
    assert!(response.is_ok());
    let text = response.as_mut().unwrap().bytes().await?;
    assert_eq!(
        text.len(),
        response?
            .headers()
            .get("Content-Length")?
            .unwrap()
            .parse::<usize>()
            .unwrap()
    );

    let headers: Headers = [("Authorization", "Basic 123456789==")].iter().collect();
    let request = Request::new_with_init(
        "https://reqres.in/api/users",
        RequestInit::new().method("POST").headers(headers.as_ref()),
    )?;
    let mut response = Fetch::Request(&request).fetch().await;
    assert!(response.is_ok());

    let _json: Value = response.as_mut().unwrap().json().await?;
    let text = response?.text().await;
    assert!(text.is_err()); // Body used already

    Ok(())
}
