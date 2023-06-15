use connector::CloudflareExecutor;
use hyper::Uri;
use worker::*;

mod connector;

pub use console_error_panic_hook::set_once as set_panic_hook;

#[event(fetch)]
async fn main(_req: Request, _env: Env, _ctx: Context) -> worker::Result<Response> {
    set_panic_hook();

    let client = hyper::Client::builder()
        .executor(CloudflareExecutor)
        .pool_idle_timeout(None)
        .build::<_, hyper::Body>(connector::CloudflareConnector);

    let response = client
        .get(Uri::from_static("http://example.com"))
        .await
        .map_err(map_err)?;

    let buf = hyper::body::to_bytes(response).await.map_err(map_err)?;
    let text = std::str::from_utf8(&buf).map_err(map_err)?;

    let mut response = Response::ok(text)?;
    response.headers_mut().append("Content-Type", "text/html")?;
    Ok(response)
}

fn map_err<T: std::fmt::Debug>(error: T) -> worker::Error {
    worker::Error::RustError(format!("{:?}", error))
}
