use crate::WebSocket;
use crate::{http::body::Body, HttpResponse};
use worker_sys::ext::ResponseExt;
use worker_sys::ext::ResponseInitExt;

use super::header::{header_map_from_web_sys_headers, web_sys_headers_from_header_map};

pub fn to_wasm(mut res: HttpResponse) -> web_sys::Response {
    let mut init = web_sys::ResponseInit::new();
    init.status(res.status().as_u16());
    init.headers(&web_sys_headers_from_header_map(res.headers()));
    if let Some(ws) = res.extensions_mut().remove::<WebSocket>() {
        init.websocket(ws.as_ref());
    }
    let readable_stream = res.into_body().into_inner();

    web_sys::Response::new_with_opt_readable_stream_and_init(readable_stream.as_ref(), &init)
        .unwrap()
}

pub fn from_wasm(res: web_sys::Response) -> HttpResponse {
    let mut builder =
        http::response::Builder::new().status(http::StatusCode::from_u16(res.status()).unwrap());
    header_map_from_web_sys_headers(res.headers(), builder.headers_mut().unwrap());
    if let Some(ws) = res.websocket() {
        builder = builder.extension(WebSocket::from(ws));
    }
    if let Some(body) = res.body() {
        builder.body(Body::new(body)).unwrap()
    } else {
        builder.body(Body::empty()).unwrap()
    }
}
