use worker_sys::{Schedule as EdgeSchedule};

/// [Schedule](https://developers.cloudflare.com/workers/runtime-apis/scheduled-event)
#[derive(Debug, Clone)]
pub struct Schedule {
    cron: String,
    scheduled_time: f64,
    ty: String,
    pub edge: EdgeSchedule,
    // env: Env,
}

impl From<EdgeSchedule> for Schedule {
    fn from(schedule: EdgeSchedule) -> Self {
         Self {
            cron: schedule.cron(),
            scheduled_time: schedule.scheduled_time(),
            ty: String::from("scheduled"),
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
    pub fn schedule(&self) -> f64 {
        self.scheduled_time.clone()
    }
}