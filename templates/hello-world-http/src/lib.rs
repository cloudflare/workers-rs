use worker::*;

#[event(fetch)]
async fn fetch(
    _req: HttpRequest,
    _env: Env,
    _ctx: Context,
) -> Result<HttpResponse> {
    Ok(http::Response::builder()
        .status(http::StatusCode::OK)
        .body(Body::empty())?)
}