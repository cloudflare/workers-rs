use worker_macros::event;

#[event(fetch)]
async fn fetch(a: u32) -> u32 {
    0
}

fn main() {}
