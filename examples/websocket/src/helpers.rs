use crate::error::Error;
use worker::{
    wasm_bindgen,
    wasm_bindgen::{prelude::wasm_bindgen, JsValue},
    Response,
};

#[wasm_bindgen]
extern "C" {
    type Math;
    #[wasm_bindgen(static_method_of = Math)]
    fn random() -> f64;

    #[wasm_bindgen(js_name = setTimeout)]
    pub fn set_timeout(closure: JsValue, millis: i32) -> i32;
}

pub trait IntoResponse {
    fn into_response(self) -> worker::Result<Response>;
}

impl IntoResponse for Result<Response, Error> {
    fn into_response(self) -> worker::Result<Response> {
        match self {
            Ok(res) => Ok(res),
            Err(err) => Ok(err.into_response()?),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> worker::Result<Response> {
        let (msg, status) = self.take();
        let mut res = Response::error(msg, status)?;
        let headers = res.headers_mut();
        let _ = headers.set("content-type", "application/json;charset=UTF-8");

        Ok(res)
    }
}

pub fn random_color() -> String {
    const RADIX: u32 = 16;
    const ZEROES: &str = "000000";

    let mut x = (Math::random() * f64::from(0x00ff_ffff)).round() as u32;
    let mut y = Vec::new();

    while x > 0 {
        y.push(std::char::from_digit(x % RADIX, RADIX));
        x /= RADIX;
    }

    let z: String = y.into_iter().rev().flatten().collect();

    format!("#{}{}", &ZEROES[0..(6 - z.len())], z)
}
