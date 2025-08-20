use worker::*;

#[event(fetch)]
async fn fetch(
    _req: HttpRequest,
    _env: Env,
    _ctx: Context,
) -> Result<HttpResponse> {
    console_error_panic_hook::set_once();
    Ok(http::Response::builder()
        .status(http::StatusCode::OK)
        .body(Body::empty())?)
}