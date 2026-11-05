use worker_macros::event;

#[event(fetch, entrypoint = 42)]
async fn fetch() {}

fn main() {}
