// `#[wasm_bindgen(jspi)]` is experimental and warns as deprecated.
#![allow(deprecated)]

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use worker::wasm_bindgen::prelude::*;
use worker::*;

fn main() {}

/// A synchronous `fetch` export returning a Promise through JSPI: the Tokio
/// runtime parks by suspending the Wasm stack, so stock `tokio::net` runs on
/// the host event loop with no wasm-specific async bridging.
#[wasm_bindgen(jspi)]
pub fn fetch(
    req: worker_sys::web_sys::Request,
    _env: Env,
    _ctx: worker_sys::Context,
) -> std::result::Result<worker_sys::web_sys::Response, JsValue> {
    let response = handle(Request::from(req)).unwrap_or_else(|e| {
        console_error!("{e}");
        Response::error(e.to_string(), 500).unwrap()
    });
    response
        .into_raw()
        .map_err(|e| JsValue::from(e.into().to_string()))
}

fn handle(req: Request) -> Result<Response> {
    let url = req.url()?;
    let Some(host) = url
        .query_pairs()
        .find(|(k, _)| k == "host")
        .map(|(_, v)| v.into_owned())
    else {
        return Response::ok("usage: /?host=example.com");
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| Error::RustError(e.to_string()))?;
    let body = runtime
        .block_on(async {
            let mut stream = TcpStream::connect((host.as_str(), 80)).await?;
            stream
                .write_all(format!("HEAD / HTTP/1.0\r\nHost: {host}\r\n\r\n").as_bytes())
                .await?;
            let mut out = String::new();
            stream.read_to_string(&mut out).await?;
            Ok::<_, std::io::Error>(out)
        })
        .map_err(|e| Error::RustError(e.to_string()))?;
    Response::ok(body)
}
