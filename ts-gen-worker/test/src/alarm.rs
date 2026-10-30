use crate::Env;
use ts_gen_worker::*;
use wasm_bindgen::prelude::*;

use super::SomeSharedData;

#[durable_object]
pub struct AlarmObject {
    state: DurableObjectState,
    _buffer: Vec<u8>,
}

impl DurableObject for AlarmObject {
    type Env = Env;

    fn new(state: DurableObjectState, _env: Env) -> AlarmObject {
        Self {
            state,
            _buffer: Vec::with_capacity(111_000_000),
        }
    }

    async fn fetch(&self, _req: Request) -> Result<Response> {
        self.state
            .storage()
            .set_alarm(js_sys::Date::now() + 100.0)
            .await?;
        let alarmed: bool = self
            .state
            .storage()
            .get_into("alarmed")
            .await?
            .unwrap_or(false);
        Response::builder().ok(alarmed.to_string())
    }

    async fn alarm(&self) -> Result<()> {
        self.state
            .storage()
            .put_serialized("alarmed", &true)
            .await?;
        console_log!("Alarm has been triggered!");
        Ok(())
    }
}

pub async fn handle_alarm(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    // GAP: `ALARM` is bound in wrangler.toml but not in the suite's mf.ts, and no
    // spec drives it; the route is kept for route parity only.
    let namespace: DurableObjectNamespace = env.get("ALARM")?;
    let stub = namespace.get(&namespace.id_from_name("alarm")?)?;
    stub.fetch_with_str("https://fake-host/alarm").await
}
