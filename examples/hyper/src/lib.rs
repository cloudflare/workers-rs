use std::str::Utf8Error;

use worker::*;

pub use console_error_panic_hook::set_once as set_panic_hook;

async fn make_request(
    mut sender: hyper::client::conn::SendRequest<hyper::Body>,
    request: hyper::Request<hyper::Body>,
) -> Result<Response> {
    // Send and recieve HTTP request
    let hyper_response = sender
        .send_request(request)
        .await
        .map_err(map_hyper_error)?;

    // Convert back to worker::Response
    let buf = hyper::body::to_bytes(hyper_response)
        .await
        .map_err(map_hyper_error)?;
    let text = std::str::from_utf8(&buf).map_err(map_utf8_error)?;
    let mut response = Response::ok(text)?;
    response
        .headers_mut()
        .append("Content-Type", "text/html; charset=utf-8")?;
    Ok(response)
}
#[event(fetch)]
async fn main(_req: Request, _env: Env, _ctx: Context) -> worker::Result<Response> {
    set_panic_hook();

    let socket = Socket::builder()
        .secure_transport(SecureTransport::On)
        .connect("example.com", 443)?;

    let (sender, connection) = hyper::client::conn::handshake(socket)
        .await
        .map_err(map_hyper_error)?;

    let request = hyper::Request::builder()
        .header("Host", "example.com")
        .method("GET")
        .body(hyper::Body::from(""))
        .map_err(map_hyper_http_error)?;

    tokio::select!(
        res = connection => {
            console_error!("Connection exited: {:?}", res);
            Err(worker::Error::RustError("Connection exited".to_string()))
        },
        result = make_request(sender, request) => result
    )
}

fn map_utf8_error(error: Utf8Error) -> worker::Error {
    worker::Error::RustError(format!("Utf8Error: {:?}", error))
}

fn map_hyper_error(error: hyper::Error) -> worker::Error {
    worker::Error::RustError(format!("hyper::Error: {:?}", error))
}

fn map_hyper_http_error(error: hyper::http::Error) -> worker::Error {
    worker::Error::RustError(format!("hyper::http::Error: {:?}", error))
}
