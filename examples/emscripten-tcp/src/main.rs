//! Raw TCP from a Worker with `tokio::net`: `/?host=example.com` sends
//! an HTTP HEAD request to port 80 of the host over a `TcpStream` and returns
//! the reply; `/do?host=...` does the same from inside a Durable Object.

use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use worker::*;

fn main() {}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let url = req.url()?;
    let Some(host) = url
        .query_pairs()
        .find(|(k, _)| k == "host")
        .map(|(_, v)| v.into_owned())
    else {
        return Response::ok("usage: /?host=example.com or /do?host=example.com\n");
    };
    if url.path() == "/do" {
        let stub = env
            .durable_object("PROBE")?
            .id_from_name(&host)?
            .get_stub()?;
        return stub
            .fetch_with_str(&format!("https://do/?host={host}"))
            .await;
    }
    Response::ok(head(&host).await?)
}

#[durable_object]
pub struct Probe;

impl DurableObject for Probe {
    fn new(_state: State, _env: Env) -> Self {
        Self
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let host = req
            .url()?
            .query_pairs()
            .find(|(k, _)| k == "host")
            .map(|(_, v)| v.into_owned())
            .ok_or_else(|| Error::RustError("missing host".into()))?;
        Response::ok(head(&host).await?)
    }
}

/// An HTTP HEAD request over a plain TCP connection, within 5 seconds. The
/// hostname resolves through Emscripten's asynchronous getaddrinfo
/// (emscripten-core/emscripten#27742, applied by worker-build).
async fn head(host: &str) -> Result<String> {
    let io = async {
        let mut stream = TcpStream::connect((host, 80)).await?;
        stream
            .write_all(format!("HEAD / HTTP/1.0\r\nHost: {host}\r\n\r\n").as_bytes())
            .await?;
        let mut reply = String::new();
        stream.read_to_string(&mut reply).await?;
        Ok::<_, std::io::Error>(reply)
    };
    match tokio::time::timeout(Duration::from_secs(5), io).await {
        Ok(reply) => reply.map_err(|e| Error::RustError(e.to_string())),
        Err(_) => Err(Error::RustError("timed out".into())),
    }
}
