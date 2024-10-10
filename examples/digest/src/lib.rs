use worker::{
    crypto::{DigestStream, DigestStreamAlgorithm},
    wasm_bindgen::prelude::*,
    worker_sys::web_sys,
    *,
};

#[event(fetch)]
async fn main(_req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let payload = "Hello, World!";
    let expected = "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f";

    let read_stream = str_to_readable_stream(payload);
    let digest_stream = DigestStream::new(DigestStreamAlgorithm::Sha256);

    // just kick the promise off to the background, we'll await the digest itself
    let _ = read_stream.pipe_to(digest_stream.raw());

    let bytes = digest_stream.digest().await?.to_vec();
    let output = hex::encode(bytes);

    if output != expected {
        Response::ok(format!(
            "UHOH! Hash of '{payload}' is: {output}, but expected: {expected}!"
        ))
    } else {
        Response::ok(format!("Woohoo! Hash of '{payload}' is: {output}"))
    }
}

fn str_to_readable_stream(value: &str) -> web_sys::ReadableStream {
    let req_init = web_sys::RequestInit::new();
    req_init.set_method("POST");
    req_init.set_body(&JsValue::from_str(value));
    let req = web_sys::Request::new_with_str_and_init("http://internal", &req_init).unwrap();
    req.body().unwrap()
}
