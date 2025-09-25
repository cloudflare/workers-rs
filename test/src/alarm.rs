use std::time::Duration;
use worker::{console_log, durable_object, wasm_bindgen, Env, Request, Response, Result, State};

use super::SomeSharedData;

#[durable_object]
pub struct AlarmObject {
    state: State,
    // used for memory leak detection
    _buffer: Vec<u8>,
}

impl DurableObject for AlarmObject {
    fn new(state: State, _: Env) -> Self {
        Self {
            state,
            _buffer: Vec::with_capacity(111_000_000),
        }
    }

    async fn fetch(&self, _: Request) -> Result<Response> {
        self.state
            .storage()
            .set_alarm(Duration::from_millis(100))
            .await?;
        let alarmed: bool = match self.state.storage().get("alarmed").await {
            Ok(alarmed) => alarmed,
            Err(e) if e.to_string() == "No such value in storage." => false,
            Err(e) => return Err(e),
        };
        Response::ok(alarmed.to_string())
    }

    async fn alarm(&self) -> Result<Response> {
        self.state.storage().put("alarmed", true).await?;
        console_log!("Alarm has been triggered!");
        Response::ok("ALARMED")
    }
}

pub async fn handle_alarm(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let namespace = env.durable_object("ALARM")?;
    let stub = namespace.id_from_name("alarm")?.get_stub()?;
    // when calling fetch to a Durable Object, a full URL must be used. Alternatively, a
    // compatibility flag can be provided in wrangler.toml to opt-in to older behavior:
    // https://developers.cloudflare.com/workers/platform/compatibility-dates#durable-object-stubfetch-requires-a-full-url
    stub.fetch_with_str("https://fake-host/alarm").await
}
