use worker_macros::event;

#[event(fetch, entrypoint = "First", entrypoint = "Second")]
async fn fetch() {}

fn main() {}
