use worker_macros::event;

#[event(connect)]
async fn connect(a: u32, b: u32) -> u32 {
    0
}

fn main() {}
