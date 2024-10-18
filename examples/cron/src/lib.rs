use worker::*;

#[event(scheduled)]
async fn scheduled(_evt: ScheduledEvent, _env: Env, _ctx: ScheduleContext) {
    wasm_rs_dbg::dbg!("Hello cron!");
}
