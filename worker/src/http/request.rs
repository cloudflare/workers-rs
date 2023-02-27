use bytes::Buf;
use futures_util::StreamExt;
use wasm_bindgen::JsCast;
use worker_sys::ext::{HeadersExt, RequestExt};

use crate::{AbortSignal, Cf};

use crate::body::{Body, WasmStreamBody};

fn version_from_string(version: &str) -> http::Version {
    match version {
        "HTTP/0.9" => http::Version::HTTP_09,
        "HTTP/1.0" => http::Version::HTTP_10,
        "HTTP/1.1" => http::Version::HTTP_11,
        "HTTP/2.0" => http::Version::HTTP_2,
        "HTTP/3.0" => http::Version::HTTP_3,
        _ => unreachable!("no other versions exist"),
    }
}

pub fn from_wasm(req: web_sys::Request) -> http::Request<Body> {
    let mut builder = http::Request::builder()
        .method(&*req.method())
        .uri(req.url());

    if let Some(cf) = req.cf() {
        builder = builder
            .version(version_from_string(&cf.http_protocol()))
            .extension(Cf::from(cf));
    }

    for header in req.headers().entries() {
        let header = header.unwrap().unchecked_into::<js_sys::Array>();
        builder = builder.header(
            header.get(0).as_string().unwrap(),
            header.get(1).as_string().unwrap(),
        );
    }

    let body = req
        .body()
        .map(|body| {
            WasmStreamBody::new(
                wasm_streams::ReadableStream::from_raw(body.dyn_into().unwrap()).into_stream(),
            )
        })
        .into();

    builder.body(body).unwrap()
}

pub fn into_wasm(mut req: http::Request<Body>) -> web_sys::Request {
    let method = req.method().to_string();
    let uri = req.uri().to_string();

    let headers = web_sys::Headers::new().unwrap();
    for (name, value) in req
        .headers()
        .into_iter()
        .filter_map(|(name, value)| value.to_str().map(|value| (name.as_str(), value)).ok())
    {
        headers.append(name, value).unwrap();
    }

    let signal = req.extensions_mut().remove::<AbortSignal>();

    let body = req.into_body();
    let body = if body.is_none() {
        None
    } else {
        let stream = wasm_streams::ReadableStream::from_stream(body.map(|chunk| {
            chunk
                .map(|buf| js_sys::Uint8Array::from(buf.chunk()).into())
                .map_err(|_| wasm_bindgen::JsValue::NULL)
        }));

        Some(stream.into_raw().unchecked_into())
    };

    let mut init = web_sys::RequestInit::new();
    init.method(&method)
        .headers(&headers)
        .signal(signal.as_deref())
        .body(body.as_ref());

    web_sys::Request::new_with_str_and_init(&uri, &init).unwrap()
}
