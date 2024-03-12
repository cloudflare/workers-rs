use crate::http::body::Body;

pub fn from_wasm(_req: web_sys::Request) -> http::Request<Body> {
    todo!()
}

pub fn to_wasm(_req: http::Request<Body>) -> web_sys::Request {
    todo!()
}
