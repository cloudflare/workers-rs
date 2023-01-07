use std::time::{Duration, Instant};

use futures_channel::mpsc;
use futures_util::{SinkExt, StreamExt};
use http::StatusCode;
use reqwest::{
    blocking::{
        multipart::{Form, Part},
        Client,
    },
    redirect::Policy,
    Body, Client as AsyncClient,
};
use serde::{Deserialize, Serialize};
use util::*;

mod util;

#[test]
fn request() {
    let _ = get("request", |r| r);
}

#[test]
fn async_request() {
    let _ = get("async-request", |r| r);
}

#[test]
fn test_data() {
    let body = get("test-data", |r| r).text().unwrap();
    assert_eq!(body, "data ok");
}

#[test]
fn headers() {
    expect_wrangler();

    let response = post("headers", |r| r.header("A", "B"));
    let headers = response.headers();

    assert_eq!(headers.get("A").map(|v| v.to_str().unwrap()), Some("B"));
}

#[test]
fn is_secret() {
    let form = Form::new().text("secret", "EXAMPLE_SECRET");
    let body = post("is-secret", |r| r.multipart(form)).text().unwrap();
    assert_eq!(body, "example");
}

// This test is for the /formdata-file-size/* routes which rely on request order because some data
// gets stored in a KV store.
#[test]
fn formdata() {
    #[derive(Deserialize)]
    struct Key {
        name: String,
    }

    let bytes = b"workers-rs is cool!";
    let file_part = Part::bytes(bytes.to_vec()).file_name("file");
    let form = Form::new().part("file", file_part);

    let hashes: Vec<Key> = post("formdata-file-size", |r| r.multipart(form))
        .json()
        .unwrap();

    // Make sure the key sent back is valid. If this request fails we'll get a non 200 status code
    // and panic.
    let _ = get(&format!("formdata-file-size/{}", &hashes[0].name), |r| r);
}

#[test]
fn user_id_test() {
    // Route pattern: /user/:id/test
    let body = get("user/example/test", |r| r).text().unwrap();
    assert_eq!(body, "TEST user id: example");
}

#[test]
fn user() {
    // Route pattern: /user/:id
    let body: serde_json::Value = get("user/example", |r| r).json().unwrap();
    assert_eq!(body["id"], "example");
}

#[test]
fn post_account_id_zones() {
    // Route pattern: /account/:id/zones
    let body = post("account/example/zones", |r| r).text().unwrap();
    assert_eq!(body, "Create new zone for Account: example");
}

#[test]
fn get_account_id_zones() {
    // Route pattern: /account/:id/zones
    let body = get("account/example/zones", |r| r).text().unwrap();
    assert_eq!(
        body,
        "Account id: example..... You get a zone, you get a zone!"
    );
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
    assert_eq!(body, "received responses with codes 200 and 200");
}

#[test]
fn fetch_json() {
    let body = get("fetch_json", |r| r).text().unwrap();
    assert_eq!(
        body,
        "API Returned user: 1 with title: delectus aut autem and completed: false"
    );
}

#[test]
fn proxy_request() {
    // Because the sandbox worker passes the response without touching it, we might get a response
    // with a body thats compressed. So we'll just use this which isn't compressed and is small.
    let body = get(
        "proxy_request/https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Encoding/contributors.txt",
        |r| r,
    ).text().unwrap();

    assert!(body.contains("# Original Wiki contributors"));
}

#[test]
fn durable_id() {
    let body = get("durable/example", |r| r).text().unwrap();
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
fn some_secret() {
    let body = get("secret", |r| r).text().unwrap();
    assert_eq!(body, "secret!");
}

#[test]
fn kv_key_value() {
    #[derive(Deserialize)]
    struct Keys {
        keys: Vec<serde_json::Value>,
    }
    let keys: Keys = post("kv/a/b", |r| r).json().unwrap();
    assert!(!keys.keys.is_empty());
}

#[test]
fn bytes() {
    let bytes = get("bytes", |r| r).bytes().unwrap();
    assert_eq!(bytes.to_vec(), &[1u8, 2, 3, 4, 5, 6, 7]);
}

#[test]
fn api_data() {
    #[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
    #[serde(rename_all = "camelCase")]
    struct ApiData {
        user_id: i32,
        title: String,
        completed: bool,
    }

    let data = ApiData {
        user_id: 0,
        title: "Hi!".into(),
        completed: false,
    };

    let mut response_data: ApiData = post("api-data", |r| r.json(&data)).json().unwrap();

    // This endpoint reverses the bytes of the todo to show that id does something, so we'll just
    // flip them back and compare.
    unsafe { response_data.title.as_mut_vec().reverse() };

    assert_eq!(data, response_data);
}

#[test]
fn nonsense_repeat() {
    let body = post("nonsense-repeat", |r| r).text().unwrap();
    assert_eq!(body, "data ok");
}

#[test]
fn status_code() {
    let status_code = reqwest::blocking::get("http://127.0.0.1:8787/status/418")
        .unwrap()
        .status();
    assert_eq!(status_code, StatusCode::IM_A_TEAPOT);
}

#[test]
fn root() {
    // Theres more routes with the exact same path and respond function, so we'll just cover them
    // all with this.
    let response = put("", |r| r);
    let testing_header = response
        .headers()
        .get("x-testing")
        .cloned()
        .and_then(|x| x.to_str().ok().map(String::from))
        .expect("no testing header");

    assert_eq!(testing_header, "123");
}

#[test]
fn r#async() {
    // Theres more routes with the exact same path and respond function, so we'll just cover them
    // all with this.
    let response = put("async", |r| r);
    let testing_header = response
        .headers()
        .get("x-testing")
        .cloned()
        .and_then(|x| x.to_str().ok().map(String::from))
        .expect("no testing header");

    assert_eq!(testing_header, "123");
}

#[test]
fn catchall() {
    let body = options("Hello, world!", |r| r).text().unwrap();
    assert_eq!(body, "/Hello,%20world!");
}

#[test]
fn request_init_fetch() {
    // This route just fetches the cloudflare home page which is compressed, so we'll just assume
    //  any successful response means it worked.
    let _ = get("request-init-fetch", |r| r);
}

#[test]
fn cancelled_fetch() {
    let body = get("cancelled-fetch", |r| r).text().unwrap();
    assert!(body.starts_with("AbortError:"));
}

#[test]
fn fetch_timeout() {
    let body = get("fetch-timeout", |r| r).text().unwrap();
    assert_eq!(body, "Cancelled");
}

#[test]
fn request_init_fetch_post() {
    #[derive(Deserialize)]
    struct Data {
        url: String,
    }

    let data: Data = get("request-init-fetch-post", |r| r).json().unwrap();
    assert_eq!(data.url, "https://httpbin.org/post");
}

#[test]
fn redirect_default() {
    let client = Client::builder().redirect(Policy::none()).build().unwrap();
    let response = client
        .get("http://127.0.0.1:8787/redirect-default")
        .send()
        .expect("could not make request");
    let location = response
        .headers()
        .get("location")
        .cloned()
        .and_then(|h| h.to_str().ok().map(String::from))
        .expect("no location header");

    assert_eq!(location, "https://example.com/");
}

#[test]
fn redirect_307() {
    let client = Client::builder().redirect(Policy::none()).build().unwrap();
    let response = client
        .get("http://127.0.0.1:8787/redirect-307")
        .send()
        .expect("could not make request");
    let location = response
        .headers()
        .get("location")
        .cloned()
        .and_then(|h| h.to_str().ok().map(String::from))
        .expect("no location header");

    assert_eq!(location, "https://example.com/");
    assert_eq!(response.status(), 307);
}

#[test]
fn now() {
    // JavaScript doesn't use a date format that chrono can natively parse, so we'll just assume
    // any 200 status code is a pass.
    get("now", |r| r);
}

#[test]
fn cloned() {
    let resp = get("cloned", |r| r);
    assert_eq!(resp.text().unwrap(), "true")
}

#[test]
fn cloned_stream() {
    let resp = get("cloned-stream", |r| r);
    assert_eq!(resp.text().unwrap(), "true")
}

#[test]
fn cloned_fetch() {
    let resp = get("cloned-fetch", |r| r);
    assert_eq!(resp.text().unwrap(), "true")
}

#[test]
fn wait() {
    const MILLIS: u64 = 100;
    let then = Instant::now();
    get(&format!("wait/{MILLIS}"), |r| r);
    assert!(then.elapsed() >= Duration::from_millis(MILLIS));
}

#[test]
fn custom_response_body() {
    let body = get("custom-response-body", |r| r).bytes().unwrap();
    assert_eq!(body.to_vec(), b"hello");
}

#[test]
fn init_called() {
    // JavaScript doesn't use a date format that chrono can natively parse, so we'll just assume
    // any 200 status code is a pass.
    let body = get("init-called", |r| r).text().unwrap();
    assert_eq!(body, "true");
}

#[tokio::test]
async fn xor() {
    expect_wrangler();

    let (mut body_sink, rx) = mpsc::channel::<u8>(32);
    let req_stream = rx.map(|byte| Ok::<Vec<u8>, std::io::Error>(vec![byte]));
    let body = Body::wrap_stream(req_stream);

    let xor_num = 10;

    // We need to send a single byte for us to get the initial response.
    body_sink.send(0).await.unwrap();

    let client = AsyncClient::new();
    let mut res_stream = client
        .post(&format!("http://127.0.0.1:8787/xor/{xor_num}"))
        .body(body)
        .send()
        .await
        .expect("could not make request")
        .bytes_stream();

    // Skip that first byte we use to get the stream.
    let _ = res_stream.next().await;

    for byte in 0..=255u8 {
        body_sink.send(byte).await.unwrap();
        let xored_byte = res_stream
            .next()
            .await
            .expect("XOR stream closed unexpectedly")
            .map(|chunk| chunk[0])
            .expect("unexpected error with response stream");

        assert_eq!(xored_byte, byte ^ xor_num);
    }

    body_sink
        .close()
        .await
        .expect("unable to close body stream");

    // Ensure that closing our request stream ends the body.
    assert!(res_stream.next().await.is_none());
}

#[test]
fn cache_example() {
    // The first request should be a miss, and the request should be cached
    let body: serde_json::Value = get("cache-example", |r| r).json().unwrap();
    let expected_ts = &body["timestamp"];

    // The subsequent request should now be cache hits, so the API should return same
    // timestamp as first request
    for _ in 0..5 {
        let body: serde_json::Value = get("cache-example", |r| r).json().unwrap();
        let curr_ts = &body["timestamp"];

        assert_eq!(expected_ts, curr_ts);
    }
}

#[test]
fn cache_stream() {
    // The first request should be a miss, and the request should be cached
    let expected_body = get("cache-stream", |r| r).text().unwrap();

    // The subsequent request should now be cache hits, so the API should return same body
    for _ in 0..5 {
        let curr_body = get("cache-stream", |r| r).text().unwrap();
        assert_eq!(expected_body, curr_body);
    }
}

#[test]
fn cache_api() {
    let key = "example.org";
    let get_endpoint = format!("cache-api/get/{}", key);
    let put_endpoint = format!("cache-api/put/{}", key);
    let delete_endpoint = format!("cache-api/delete/{}", key);

    // First time should result in cache miss
    let body = get(get_endpoint.as_str(), |r| r).text().unwrap();
    assert_eq!(body, "cache miss");

    // Add key to cache
    let body: serde_json::Value = put(put_endpoint.as_str(), |r| r).json().unwrap();
    let expected_ts = &body["timestamp"];

    // Should now be cache hit
    let body: serde_json::Value = get(get_endpoint.as_str(), |r| r).json().unwrap();
    assert_eq!(expected_ts, &body["timestamp"]);

    // Delete key from cache
    let body: serde_json::Value = post(delete_endpoint.as_str(), |r| r).json().unwrap();
    assert_eq!("Success", body);

    // Make sure key is now deleted
    let body = get(get_endpoint.as_str(), |r| r).text().unwrap();
    assert_eq!(body, "cache miss");

    // Another delete should fail
    let body: serde_json::Value = post(delete_endpoint.as_str(), |r| r).json().unwrap();
    assert_eq!("ResponseNotFound", body);
}

#[test]
fn test_service_binding() {
    let body: String = get("remote-by-request", |r| r).text().unwrap();
    assert_eq!(body, "hello world");

    let body: String = get("remote-by-path", |r| r).text().unwrap();
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
