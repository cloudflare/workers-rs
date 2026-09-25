//! An inbound TCP server with `tokio::net::TcpListener`, running inside a
//! Durable Object: a line-echo that greets, echoes each line back upper-cased
//! and hangs up on `quit`.
//!
//! The Worker's `connect` handler forwards each inbound connection to the
//! object, whose `connect` handler hands it to a Tokio listener bound on the
//! same port, so an existing Tokio server accepts it as usual. A Durable
//! Object is one I/O context, so its handlers share one Tokio runtime and the
//! listener outlives any single connection; a Worker's handlers each run on a
//! runtime of their own for the duration of that connection.

use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use worker::*;

fn main() {}

/// The port `wrangler.toml` declares under `[[connect]]`.
const PORT: u16 = 7000;

/// Pipes the inbound connection to the object's server.
#[event(connect)]
async fn connect(socket: Socket, env: Env, _ctx: Context) -> Result<()> {
    let stub = env
        .durable_object("ECHO")?
        .id_from_name("echo")?
        .get_stub()?;
    let upstream = stub.connect(&format!("127.0.0.1:{PORT}"))?;
    let (mut client_read, mut client_write) = tokio::io::split(socket);
    let (mut server_read, mut server_write) = tokio::io::split(upstream);
    tokio::select! {
        r = tokio::io::copy(&mut client_read, &mut server_write) => r,
        r = tokio::io::copy(&mut server_read, &mut client_write) => r,
    }?;
    Ok(())
}

#[durable_object(connect)]
pub struct Echo {
    listening: AtomicBool,
}

impl DurableObject for Echo {
    fn new(_state: State, _env: Env) -> Self {
        Self {
            listening: AtomicBool::new(false),
        }
    }

    async fn fetch(&self, _req: Request) -> Result<Response> {
        Response::ok("tcp only\n")
    }

    /// Binds the listener on the first connection; every connection is then
    /// accepted by it.
    async fn connect(&self, socket: Socket) -> Result<()> {
        if !self.listening.swap(true, Ordering::Relaxed) {
            let listener = TcpListener::bind(("0.0.0.0", PORT))
                .await
                .map_err(|e| Error::RustError(e.to_string()))?;
            tokio::spawn(async move {
                loop {
                    match listener.accept().await {
                        Ok((stream, peer)) => {
                            tokio::spawn(async move {
                                if let Err(e) = echo(stream).await {
                                    console_error!("{peer}: {e}");
                                }
                            });
                        }
                        Err(e) => console_error!("accept failed: {e}"),
                    }
                }
            });
        }
        socket.handle_as_node_connection().await
    }
}

async fn echo(stream: TcpStream) -> std::io::Result<()> {
    let (reader, mut writer) = stream.into_split();
    writer
        .write_all(b"hello from tokio on workers; type lines, `quit` to leave\n")
        .await?;
    let mut lines = BufReader::new(reader).lines();
    while let Some(line) = lines.next_line().await? {
        if line.trim() == "quit" {
            writer.write_all(b"bye\n").await?;
            break;
        }
        writer.write_all(line.to_uppercase().as_bytes()).await?;
        writer.write_all(b"\n").await?;
    }
    Ok(())
}
