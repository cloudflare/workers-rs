use worker_macros::event;

#[event(scheduled, entrypoint = "Scheduled")]
async fn scheduled() {}

fn main() {}
