use crate::http::body::Body;
use crate::Cf;
use crate::{http::redirect::RequestRedirect, AbortSignal};
use worker_sys::ext::RequestExt;

use crate::http::header::{header_map_from_web_sys_headers, web_sys_headers_from_header_map};

fn version_from_string(version: &str) -> http::Version {
    match version {
        "HTTP/0.9" => http::Version::HTTP_09,
        "HTTP/1.0" => http::Version::HTTP_10,
        "HTTP/1.1" => http::Version::HTTP_11,
        "HTTP/2" => http::Version::HTTP_2,
        "HTTP/3" => http::Version::HTTP_3,
        _ => unreachable!("no other versions exist"),
    }
}

pub fn from_wasm(req: web_sys::Request) -> http::Request<Body> {
    let mut builder = http::request::Builder::new()
        .uri(req.url())
        .extension(AbortSignal::from(req.signal()))
        .extension(RequestRedirect::from(req.redirect()))
        .method(&*req.method());

    header_map_from_web_sys_headers(req.headers(), builder.headers_mut().unwrap());

    if let Some(cf) = req.cf() {
        builder = builder
            .version(version_from_string(&cf.http_protocol()))
            .extension(Cf::new(cf));
    }

    if let Some(body) = req.body() {
        builder.body(Body::new(body)).unwrap()
    } else {
        builder.body(Body::empty()).unwrap()
    }
}

pub fn to_wasm(mut req: http::Request<Body>) -> web_sys::Request {
    let mut init = web_sys::RequestInit::new();
    init.method(req.method().as_str());
    init.headers(&web_sys_headers_from_header_map(req.headers()));
    let uri = req.uri().to_string();

    let signal = req.extensions_mut().remove::<AbortSignal>();
    init.signal(signal.as_ref().map(|s| s.inner()));

    if let Some(redirect) = req.extensions_mut().remove::<RequestRedirect>() {
        init.redirect(redirect.into());
    }

    if let Some(cf) = req.extensions_mut().remove::<Cf>() {
        // TODO: this should be handled in worker-sys
        let r = ::js_sys::Reflect::set(
            init.as_ref(),
            &wasm_bindgen::JsValue::from("cf"),
            &wasm_bindgen::JsValue::from(cf.inner()),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
    }

    if let Some(readable_stream) = req.into_body().into_inner() {
        init.body(Some(readable_stream.as_ref()));
    }

    web_sys::Request::new_with_str_and_init(&uri, &init).unwrap()
}
