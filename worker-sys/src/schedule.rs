use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=ScheduledEvent)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type ScheduledEvent;

    #[wasm_bindgen(structural, method, getter, js_class=ScheduledEvent, js_name=scheduledTime)]
    pub fn scheduled_time(this: &ScheduledEvent) -> f64;

    #[wasm_bindgen(structural, method, getter, js_class=ScheduledEvent, js_name=cron)]
    pub fn cron(this: &ScheduledEvent) -> String;
}

/// [Context](https://developers.cloudflare.com/workers/runtime-apis/scheduled-event#syntax-module-worker)
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=ScheduleContext)]
    #[derive(Debug, Clone)]
    pub type ScheduleContext;

    #[wasm_bindgen(structural, method, js_class=ScheduleContext, js_name=waitUntil)]
    pub fn wait_until(this: &ScheduleContext, promise: js_sys::Promise);
}
