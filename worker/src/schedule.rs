use worker_sys::{ScheduledEvent as EdgeScheduledEvent};
use wasm_bindgen::prelude::*;

/// [Schedule](https://developers.cloudflare.com/workers/runtime-apis/scheduled-event#syntax-module-worker)
#[derive(Debug, Clone)]
pub struct ScheduledEvent {
    cron: String,
    scheduled_time: f64,
    ty: String,
}

impl From<EdgeScheduledEvent> for ScheduledEvent {
    fn from(schedule: EdgeScheduledEvent) -> Self {
        Self {
            cron: schedule.cron(),
            scheduled_time: schedule.scheduled_time(),
            ty: String::from("scheduled"),
        }
    }
}

impl ScheduledEvent {
    /// get cron trigger
    pub fn cron(&self) -> String {
        self.cron.clone()
    }

    /// get type
    pub fn ty(&self) -> String {
        self.ty.clone()
    }

    /// get trigger time as f64
    pub fn schedule(&self) -> f64 {
        self.scheduled_time.clone()
    }
}


/// [Context](https://developers.cloudflare.com/workers/runtime-apis/scheduled-event#syntax-module-worker)
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=ScheduleContext)]
    #[derive(Debug)]
    pub type ScheduleContext;

    #[wasm_bindgen(structural, method, js_class=ScheduleContext, js_name=waitUntil)]
    pub fn wait_until(this: &ScheduleContext, promise: js_sys::Promise);
}