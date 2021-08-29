use worker::{durable::State, *};

#[durable_object]
pub struct Counter {
    count: usize,
    state: State,
    initialized: bool,
    env: std::sync::Arc<Env>,
}

#[durable_object]
impl DurableObject for Counter {
    fn new(state: worker::durable::State, env: worker::Env) -> Self {
        Self {
            count: 0,
            initialized: false,
            state,
            env: std::sync::Arc::new(env),
        }
    }

    async fn fetch(&mut self, _req: worker::Request) -> worker::Result<worker::Response> {
        if !self.initialized {
            self.initialized = true;
            self.count = self.state.storage().get("count").await.unwrap_or(0);
        }

        self.count += 10;
        self.state.storage().put("count", self.count).await?;
        Response::ok(&format!(
            "count: {}, secret: {}",
            self.count.to_string(),
            &self.env.secret("SOME_SECRET")?.to_string()
        ))
    }
}
