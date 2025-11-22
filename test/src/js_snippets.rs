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

#[wasm_bindgen(
    inline_js = "export function js_throw_error() { throw new Error('Intentional JS error for testing recovery'); }"
)]
extern "C" {
    fn js_throw_error();
}

#[worker::send]
pub async fn performance_now(_req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    Response::ok(format!("now: {}", js_performance_now()))
}

#[worker::send]
pub async fn console_log(_req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    js_console_log("test".to_owned());
    Response::ok("OK")
}

pub fn throw_js_error(_req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    // This will call the JS function which throws an error
    js_throw_error();
    Response::ok("This should never be reached")
}
