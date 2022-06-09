use chrono::Utc;
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

    async fn fetch(&mut self, req: Request) -> Result<Response> {
        if !self.initialized {
            self.initialized = true;
            self.count = self.state.storage().get("count").await.unwrap_or(0);
        }

        self.count += 10;
        self.state.storage().put("count", self.count).await?;

        if req.path().contains("alarm") {
            // Set an alarm to trigger in 500 ms:
            let now = Utc::now();
            self.state
                .storage()
                .set_alarm(now + chrono::Duration::milliseconds(500))
                .await?;
        }

        Response::ok(&format!(
            "[durable_object]: self.count: {}, secret value: {}",
            self.count,
            self.env.secret("SOME_SECRET")?.to_string()
        ))
    }

    async fn alarm(&mut self) -> Result<Response> {
        self.count = 32;
        self.state.storage().put("count", 32).await?;

        Response::ok("ALARMED")
    }
}
