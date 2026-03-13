use worker_macros::event;

#[event(scheduled)]
async fn scheduled() {}

fn main() {}
