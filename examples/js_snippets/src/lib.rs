use wasm_bindgen::prelude::*;
use worker::*;

pub use console_error_panic_hook::set_once as set_panic_hook;

#[event(fetch)]
async fn main(_req: Request, _env: Env, _ctx: Context) -> worker::Result<Response> {
    set_panic_hook();

    let now = performance_now();
    console_log(now);

    Response::ok(format!("now: {}", now))
}

#[wasm_bindgen(inline_js = "export function js_performance_now() { return performance.now(); }")]
extern "C" {
    fn js_performance_now() -> f64;
}

#[wasm_bindgen(
    inline_js = "export function js_console_log(value) { console.log(\"first log:\", value); }"
)]
extern "C" {
    fn js_console_log(value: f64);
}

pub fn performance_now() -> f64 {
    js_performance_now()
}

pub fn console_log(value: f64) {
    js_console_log(value)
}
