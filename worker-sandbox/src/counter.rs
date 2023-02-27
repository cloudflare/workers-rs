use worker::*;

#[durable_object]
pub struct Counter {
    count: usize,
    state: State,
    initialized: bool,
    env: Env,
}

#[durable_object]
impl DurableObject for Counter {
    fn new(state: State, env: Env) -> Self {
        Self {
            count: 0,
            initialized: false,
            state,
            env,
        }
    }

    async fn fetch(
        &mut self,
        _req: http::Request<body::Body>,
    ) -> Result<http::Response<body::Body>> {
        if !self.initialized {
            self.initialized = true;
            self.count = self.state.storage().get("count").await.unwrap_or(0);
        }

        self.count += 10;
        self.state.storage().put("count", self.count).await?;

        Ok(http::Response::new(
            format!(
                "[durable_object]: self.count: {}, secret value: {}",
                self.count,
                self.env.secret("SOME_SECRET")?.to_string()
            )
            .into(),
        ))
    }
}
