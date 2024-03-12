use crate::{http::body::Body, HttpResponse};

use super::header::{header_map_from_web_sys_headers, web_sys_headers_from_header_map};

pub fn to_wasm(resp: HttpResponse) -> web_sys::Response {
    let mut init = web_sys::ResponseInit::new();
    init.status(resp.status().as_u16());
    init.headers(&web_sys_headers_from_header_map(resp.headers()));

    let readable_stream = resp.into_body().into_inner();

    web_sys::Response::new_with_opt_readable_stream_and_init(readable_stream.as_ref(), &init)
        .unwrap()
}

pub fn from_wasm(resp: web_sys::Response) -> HttpResponse {
    let mut builder =
        http::response::Builder::new().status(http::StatusCode::from_u16(resp.status()).unwrap());
    header_map_from_web_sys_headers(resp.headers(), builder.headers_mut().unwrap());
    builder.body(Body::new(resp.body().unwrap())).unwrap()
}
