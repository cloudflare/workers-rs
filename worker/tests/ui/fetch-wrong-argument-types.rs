use worker::event;

#[event(fetch)]
async fn fetch(_a: String, _b: String, _c: String) -> Result<worker::Response, worker::Error> {
    Ok(worker::Response::empty().unwrap())
}

fn main() {}
