use wasm_bindgen::prelude::*;
use worker::*;

use crate::SomeSharedData;

#[wasm_bindgen(inline_js = "export function js_performance_now() { return performance.now(); }")]
extern "C" {
    fn js_performance_now() -> f64;
}

#[wasm_bindgen(
    inline_js = "export function js_console_log(value) { console.log(\"first log:\", value); }"
)]
extern "C" {
    fn js_console_log(value: String);
}

pub async fn performance_now(_req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    Response::ok(format!("now: {}", js_performance_now()))
}

pub async fn console_log(_req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    js_console_log("test".to_owned());
    Response::ok("OK")
}
