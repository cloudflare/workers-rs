use crate::http::body::Body;

pub fn to_wasm(_resp: http::Response<Body>) -> web_sys::Response {
    todo!()
}

pub fn from_wasm(_resp: web_sys::Response) -> http::Response<Body> {
    todo!()
}
