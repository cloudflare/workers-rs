use std::time::Duration;
use worker::{
    console_log, durable_object, js_sys, wasm_bindgen, DurableObject, Env, GetAlarmOptions,
    Request, Response, Result, SetAlarmOptions, State,
};

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

    async fn fetch(&self, req: Request) -> Result<Response> {
        if req.path() == "/alarm-options" {
            return self.alarm_options().await;
        }
        self.state
            .storage()
            .set_alarm(Duration::from_millis(100))
            .await?;
        let alarmed: bool = self.state.storage().get("alarmed").await?.unwrap_or(false);
        Response::ok(alarmed.to_string())
    }

    async fn alarm(&self) -> Result<Response> {
        self.state.storage().put("alarmed", true).await?;
        console_log!("Alarm has been triggered!");
        Response::ok("ALARMED")
    }
}

impl AlarmObject {
    // set_alarm_with_options / get_alarm_with_options / delete_alarm_with_options accept
    // GetAlarmOptions / SetAlarmOptions; the last two fields are the wire shape the runtime
    // receives (camelCase keys, as workerd's GetAlarmOptions / SetAlarmOptions JSG structs
    // read them).
    async fn alarm_options(&self) -> Result<Response> {
        let storage = self.state.storage();
        let set_opts = SetAlarmOptions {
            allow_concurrency: Some(true),
            allow_unconfirmed: Some(true),
        };
        let get_opts = GetAlarmOptions {
            allow_concurrency: Some(true),
        };
        storage
            .set_alarm_with_options(Duration::from_secs(60), set_opts.clone())
            .await?;
        let scheduled = storage
            .get_alarm_with_options(get_opts.clone())
            .await?
            .is_some();
        storage.delete_alarm_with_options(set_opts.clone()).await?;
        let after_delete = storage.get_alarm_with_options(get_opts.clone()).await?;
        let get_wire = js_sys::JSON::stringify(&serde_wasm_bindgen::to_value(&get_opts)?)?;
        let set_wire = js_sys::JSON::stringify(&serde_wasm_bindgen::to_value(&set_opts)?)?;
        Response::ok(format!(
            "{scheduled}:{after_delete:?}:{}:{}",
            String::from(get_wire),
            String::from(set_wire)
        ))
    }
}

#[worker::send]
pub async fn handle_alarm_options(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let namespace = env.durable_object("ALARM")?;
    let stub = namespace.id_from_name("alarm-options")?.get_stub()?;
    stub.fetch_with_str("https://fake-host/alarm-options").await
}

#[worker::send]
pub async fn handle_alarm(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let namespace = env.durable_object("ALARM")?;
    let stub = namespace.id_from_name("alarm")?.get_stub()?;
    // when calling fetch to a Durable Object, a full URL must be used. Alternatively, a
    // compatibility flag can be provided in wrangler.toml to opt-in to older behavior:
    // https://developers.cloudflare.com/workers/platform/compatibility-dates#durable-object-stubfetch-requires-a-full-url
    stub.fetch_with_str("https://fake-host/alarm").await
}
