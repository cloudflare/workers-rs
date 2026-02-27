use worker::event;

#[event(scheduled)]
async fn scheduled(_a: String, _b: String, _c: String) {}

fn main() {}
