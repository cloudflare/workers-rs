use cf::durable_object;
use worker::{durable::State, prelude::*};

const ONE_HOUR: u64 = 3600000;

#[durable_object]
pub struct Counter {
    count: usize,
    state: State,
    initialized: bool,
    last_backup: Date,
}

#[durable_object]
impl DurableObject for Counter {
    fn constructor(state: worker::durable::State, _env: worker::Env) -> Self {
        Self {
            count: 0,
            initialized: false,
            state,
            last_backup: Date::now(),
        }
    }

    async fn fetch(&mut self, _req: worker::Request) -> worker::Result<worker::Response> {
        // Get info from last backup
        if !self.initialized {
            self.initialized = true;
            self.count = self.state.storage().get("count").await.unwrap_or(0);
        }

        // Do a backup every hour
        if Date::now().as_millis() - self.last_backup.as_millis() > ONE_HOUR {
            self.last_backup = Date::now();
            self.state.storage().put("count", self.count).await?;
        }

        self.count += 1;
        Response::ok(self.count.to_string())
    }
}
