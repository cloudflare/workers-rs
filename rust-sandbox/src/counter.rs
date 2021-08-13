use worker::{durable::State, *};

#[durable_object]
pub struct Counter {
    count: usize,
    state: State,
    initialized: bool,
}

#[durable_object]
impl DurableObject for Counter {
    fn new(state: worker::durable::State, _env: worker::Env) -> Self {
        Self {
            count: 0,
            initialized: false,
            state,
        }
    }

    async fn fetch(&mut self, _req: worker::Request) -> worker::Result<worker::Response> {
        // Get info from last backup
        if !self.initialized {
            self.initialized = true;
            self.count = self.state.storage().get("count").await.unwrap_or(0);
        }

        self.count += 10;
        self.state.storage().put("count", self.count).await?;
        Response::ok(self.count.to_string())
    }
}
