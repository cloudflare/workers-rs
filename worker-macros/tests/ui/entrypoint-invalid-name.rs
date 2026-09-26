use worker_macros::event;

#[event(fetch, entrypoint = "not-an-identifier")]
async fn fetch() {}

#[event(fetch, entrypoint = "default")]
async fn default_handler() {}

fn main() {}
