use worker_macros::event;

#[event(scheduled)]
fn scheduled(a: u32, b: u32, c: u32) {}

fn main() {}
