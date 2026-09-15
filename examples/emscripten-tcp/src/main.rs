use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use worker::*;

fn main() {}

async fn head_request(host: &str) -> std::io::Result<String> {
    if host == "sleep" {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        return Ok("slept".into());
    }
    let mut stream = TcpStream::connect((host, 80)).await?;
    stream
        .write_all(format!("HEAD / HTTP/1.0\r\nHost: {host}\r\n\r\n").as_bytes())
        .await?;
    let mut out = String::new();
    stream.read_to_string(&mut out).await?;
    Ok(out)
}

/// Stock `tokio::net` and `tokio::time` on Workers. Under `--tokio` the
/// handler already runs on Tokio's event-loop runtime; under `--tokio=jspi`
/// it blocks on its own current-thread runtime, which parks by suspending the
/// Wasm stack.
async fn head(host: &str) -> Result<String> {
    #[cfg(worker_tokio = "jspi")]
    let body = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| Error::RustError(e.to_string()))?
        .block_on(head_request(host));
    #[cfg(not(worker_tokio = "jspi"))]
    let body = head_request(host).await;
    body.map_err(|e| Error::RustError(e.to_string()))
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let url = req.url()?;
    let Some(host) = url
        .query_pairs()
        .find(|(k, _)| k == "host")
        .map(|(_, v)| v.into_owned())
    else {
        return Response::ok("usage: /?host=example.com or /do?host=example.com");
    };
    if url.path() == "/do" {
        let stub = env.durable_object("PROBE")?.id_from_name(&host)?.get_stub()?;
        return stub.fetch_with_str(&format!("https://do/?host={host}")).await;
    }
    Response::ok(head(&host).await?)
}

/// The same request from inside a Durable Object activation.
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
