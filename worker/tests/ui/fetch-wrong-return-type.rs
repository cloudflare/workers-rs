use worker::event;

#[event(fetch)]
async fn fetch(
    _request: worker::Request,
    _env: worker::Env,
    _context: worker::Context,
) -> Result<String, String> {
    Ok(String::new())
}

fn main() {}
