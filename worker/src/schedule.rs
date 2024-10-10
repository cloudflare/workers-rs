use std::future::Future;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use worker_sys::{ScheduleContext as EdgeScheduleContext, ScheduledEvent as EdgeScheduledEvent};

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
            cron: schedule.cron().unwrap(),
            scheduled_time: schedule.scheduled_time().unwrap(),
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
        self.scheduled_time
    }
}

#[derive(Clone)]
pub struct ScheduleContext {
    edge: EdgeScheduleContext,
}

impl From<EdgeScheduleContext> for ScheduleContext {
    fn from(edge: EdgeScheduleContext) -> Self {
        Self { edge }
    }
}

impl ScheduleContext {
    pub fn wait_until<T>(&self, handler: T)
    where
        T: Future<Output = ()> + 'static,
    {
        self.edge
            .wait_until(future_to_promise(async {
                handler.await;
                Ok(JsValue::null())
            }))
            .unwrap()
    }
}
