use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type ScheduledEvent;

    #[wasm_bindgen(method, catch, getter, js_name=scheduledTime)]
    pub fn scheduled_time(this: &ScheduledEvent) -> Result<f64, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn cron(this: &ScheduledEvent) -> Result<String, JsValue>;
}

/// [Context](https://developers.cloudflare.com/workers/runtime-apis/scheduled-event#syntax-module-worker)
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone)]
    pub type ScheduleContext;

    #[wasm_bindgen(method, catch, js_name=waitUntil)]
    pub fn wait_until(this: &ScheduleContext, promise: js_sys::Promise) -> Result<(), JsValue>;
}
