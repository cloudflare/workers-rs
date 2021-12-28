use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=Schedule)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Schedule;

    #[wasm_bindgen(structural, method, getter, js_class=Schedule, js_name=scheduledTime)]
    pub fn scheduled_time(this: &Schedule) -> f64;

    #[wasm_bindgen(structural, method, getter, js_class=Schedule, js_name=cron)]
    pub fn cron(this: &Schedule) -> String;

    #[wasm_bindgen(structural, method, structural, js_class=Schedule, js_name=waitUntil)]
    pub fn wait_until(this: &Schedule, promise: &js_sys::Promise);
}