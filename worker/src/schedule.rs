use std::fmt::Error;
use std::future::Future;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::{future_to_promise};
use worker_sys::{Schedule as EdgeSchedule};

/// [Schedule](https://developers.cloudflare.com/workers/runtime-apis/scheduled-event)
#[derive(Debug, Clone)]
pub struct Schedule {
    cron: String,
    schedule: u64,
    ty: String,
    pub edge: EdgeSchedule,
    // env: Env,
}

impl From<EdgeSchedule> for Schedule {
    fn from(schedule: EdgeSchedule) -> Self {
        Self {
            cron: schedule.cron(),
            schedule: schedule.schedule(),
            ty: schedule.ty(),
            edge: schedule,
        }
    }
}

impl From<Schedule> for EdgeSchedule {
    fn from(schedule: Schedule) -> Self {
        schedule.edge.into()
    }
}

impl Schedule {
    /// get cron trigger
    pub fn cron(&self) -> String {
        self.cron.clone()
    }

    /// get type
    pub fn ty(&self) -> String {
        self.ty.clone()
    }

    /// get trigger time as u64
    pub fn schedule(&self) -> u64 {
        self.schedule.clone()
    }

    /// wrap for waitUntil
    pub fn wait_until<T>(&self, future: T)
        where
            T: Future<Output=Result<JsValue, JsValue>> + 'static
    {
        self.edge.wait_until(&future_to_promise(future))
    }
}