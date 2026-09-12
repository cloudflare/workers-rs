use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use worker::*;

fn main() {}

/// Stock `tokio::net` under JSPI: the current-thread runtime parks by
/// suspending the Wasm stack, so blocking waits run on the host event loop.
fn head(host: &str) -> Result<String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| Error::RustError(e.to_string()))?;
    runtime
        .block_on(async {
            let mut stream = TcpStream::connect((host, 80)).await?;
            stream
                .write_all(format!("HEAD / HTTP/1.0\r\nHost: {host}\r\n\r\n").as_bytes())
                .await?;
            let mut out = String::new();
            stream.read_to_string(&mut out).await?;
            Ok::<_, std::io::Error>(out)
        })
        .map_err(|e| Error::RustError(e.to_string()))
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
    Response::ok(head(&host)?)
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
        Response::ok(head(&host)?)
    }
}
