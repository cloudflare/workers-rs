use worker::*;

#[event(scheduled)]
async fn scheduled(_evt: ScheduledEvent, _env: Env, _ctx: ScheduleContext) {
    console_log!("Hello cron!");
}
