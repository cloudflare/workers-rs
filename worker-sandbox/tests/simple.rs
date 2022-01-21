use util::*;

mod util;

#[test]
fn request() {
    let _ = get("request");
}

#[test]
fn async_request() {
    let _ = get("async-request");
}

#[test]
fn test_data() {
    let body = get("test-data");
    assert_eq!(body, "data ok");
}

#[test]
fn user_id_test() {
    // Route pattern: /user/:id/test
    let body = get("user/example/test");
    assert_eq!(body, "TEST user id: example");
}

#[test]
fn user() {
    // Route pattern: /user/:id
    let body: serde_json::Value = get_json("user/example");
    assert_eq!(body["id"], "example");
}
