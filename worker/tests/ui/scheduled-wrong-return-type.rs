use worker::event;

#[event(scheduled)]
async fn scheduled(
    _event: worker::ScheduledEvent,
    _env: worker::Env,
    _context: worker::ScheduleContext,
) -> String {
}

fn main() {}
