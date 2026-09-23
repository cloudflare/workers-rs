//! Raw TCP from a Worker with stock `tokio::net`. The Worker is a small TCP
//! proxy: whatever is POSTed to `/?host=1.1.1.1&port=80` is written to that
//! address and the reply is returned; `/do?...` does the same from inside a
//! Durable Object.

use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use worker::*;

fn main() {}

struct Target {
    host: String,
    port: u16,
}

impl Target {
    fn from_url(url: &Url) -> Result<Self> {
        let get = |key| url.query_pairs().find(|(k, _)| k == key).map(|(_, v)| v);
        let host = get("host")
            .ok_or_else(|| Error::RustError("missing ?host=".into()))?
            .into_owned();
        let port = match get("port") {
            Some(p) => p
                .parse()
                .map_err(|_| Error::RustError(format!("bad port {p}")))?,
            None => 80,
        };
        Ok(Self { host, port })
    }

    /// Sends `payload` and reads until the peer closes, within 5 seconds.
    async fn exchange(&self, payload: &[u8]) -> Result<Vec<u8>> {
        let io = async {
            let mut stream = TcpStream::connect((self.host.as_str(), self.port)).await?;
            stream.write_all(payload).await?;
            let mut reply = Vec::new();
            stream.read_to_end(&mut reply).await?;
            Ok::<_, std::io::Error>(reply)
        };
        match tokio::time::timeout(Duration::from_secs(5), io).await {
            Ok(reply) => reply.map_err(|e| Error::RustError(e.to_string())),
            Err(_) => Err(Error::RustError("timed out".into())),
        }
    }
}

/// The payload to send: the POST body, or an HTTP HEAD request for a GET.
async fn payload(req: &mut Request, target: &Target) -> Result<Vec<u8>> {
    match req.method() {
        Method::Post => req.bytes().await,
        _ => Ok(format!("HEAD / HTTP/1.0\r\nHost: {}\r\n\r\n", target.host).into_bytes()),
    }
}

#[event(fetch)]
async fn fetch(mut req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let url = req.url()?;
    if !url.query().is_some_and(|q| q.contains("host=")) {
        return Response::ok(
            "GET  /?host=1.1.1.1[&port=80]   HEAD request to the host, reply returned\n\
             POST /?host=...                 body sent verbatim, reply returned\n\
             .../do?host=...                 the same from inside a Durable Object\n",
        );
    }
    if url.path() == "/do" {
        let stub = env
            .durable_object("PROXY")?
            .id_from_name("proxy")?
            .get_stub()?;
        return stub.fetch_with_request(req).await;
    }
    let target = Target::from_url(&url)?;
    let payload = payload(&mut req, &target).await?;
    Response::from_bytes(target.exchange(&payload).await?)
}

#[durable_object]
pub struct Proxy;

impl DurableObject for Proxy {
    fn new(_state: State, _env: Env) -> Self {
        Self
    }

    async fn fetch(&self, mut req: Request) -> Result<Response> {
        let target = Target::from_url(&req.url()?)?;
        let payload = payload(&mut req, &target).await?;
        Response::from_bytes(target.exchange(&payload).await?)
    }
}
