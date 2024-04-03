use worker::{
    digest_stream::{DigestStream, DigestStreamAlgorithm},
    wasm_bindgen::prelude::*,
    worker_sys::web_sys,
    *,
};

#[event(fetch)]
async fn main(_req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let payload = "Hello, World!";
    let expected = "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f";
    let bytes = hash(payload).await?;
    let output = hex::encode(bytes);
    if output != expected {
        Response::ok(format!(
            "UHOH! Hash of '{payload}' is: {output}, but expected: {expected}!"
        ))
    } else {
        Response::ok(format!(
            "Woohoo! Hash of '{payload}' is: {output}, as expected!"
        ))
    }
}

async fn hash(value: &str) -> Result<Vec<u8>> {
    let digest_stream = DigestStream::new(DigestStreamAlgorithm::Sha256);
    let mut req_init = web_sys::RequestInit::new();
    req_init.method("POST");
    req_init.body(Some(&JsValue::from_str(value)));
    let req = web_sys::Request::new_with_str_and_init("http://internal", &req_init).unwrap();
    let body = req.body().unwrap();
    // just kick the promise off to the background, we'll await the digest itself
    let _ = body.pipe_to(&digest_stream);
    Ok(digest_stream.digest().await?.to_vec())
}
