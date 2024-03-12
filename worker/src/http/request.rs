use crate::http::body::Body;

use crate::http::header::{header_map_from_web_sys_headers, web_sys_headers_from_header_map};

pub fn from_wasm(req: web_sys::Request) -> http::Request<Body> {
    let mut builder = http::request::Builder::new()
        .uri(req.url())
        .method(req.method().as_str());

    header_map_from_web_sys_headers(req.headers(), builder.headers_mut().unwrap());

    if let Some(body) = req.body() {
        builder.body(Body::new(body)).unwrap()
    } else {
        builder.body(Body::empty()).unwrap()
    }
}

pub fn to_wasm(req: http::Request<Body>) -> web_sys::Request {
    let mut init = web_sys::RequestInit::new();
    init.method(req.method().as_str());
    init.headers(&web_sys_headers_from_header_map(req.headers()));
    let uri = req.uri().to_string();

    if let Some(readable_stream) = req.into_body().into_inner() {
        init.body(Some(readable_stream.as_ref()));
    }

    web_sys::Request::new_with_str_and_init(&uri, &init).unwrap()
}
