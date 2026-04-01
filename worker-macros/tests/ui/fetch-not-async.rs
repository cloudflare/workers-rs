use worker_macros::event;

#[event(fetch)]
fn fetch(a: u32, b: u32, c: u32) -> u32 {
    0
}

fn main() {}
