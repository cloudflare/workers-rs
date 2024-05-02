use worker::*;

#[event(fetch)]
async fn fetch(
    _req: Request,
    _env: Env,
    _ctx: Context,
) -> Result<Response> {
    console_error_panic_hook::set_once();
    Response::ok("Hello World!")
}