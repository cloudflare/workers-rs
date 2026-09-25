//! An inbound TCP server with `tokio::net::TcpListener`: a line-echo that
//! greets, echoes each line back upper-cased and hangs up on `quit`.
//!
//! Two tiers of Tokio runtime mirror the platform. The thread's shared
//! (ambient) event loop hosts only the listener and its accept loop, doing no
//! I/O of its own, like a Worker's top level. Each accepted connection is
//! handed to an event loop of its own, created inside the `connect` invocation
//! that delivered the connection, so all of that connection's I/O runs within
//! its own request.

use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::tokio::schedule_isolated;
use worker::{console_error, worker_sys, Env, Socket};

fn main() {}

/// The port `wrangler.toml` declares under `[[connect]]`.
const PORT: u16 = 7000;

static LISTENING: AtomicBool = AtomicBool::new(false);

/// Routes each inbound connection to the listener, binding it first. Runs on
/// the ambient event loop, so the listener outlives any one connection;
/// worker-build exports it as the entrypoint's `connect` handler.
#[wasm_bindgen(experimental_tokio)]
pub async fn connect(
    socket: worker_sys::Socket,
    _env: Env,
    _ctx: JsValue,
) -> std::result::Result<(), JsValue> {
    if !LISTENING.swap(true, Ordering::Relaxed) {
        let listener = TcpListener::bind(("0.0.0.0", PORT))
            .await
            .map_err(|e| js_sys::Error::new(&e.to_string()))?;
        tokio::spawn(accept_loop(listener));
    }
    Socket::from(socket)
        .handle_as_node_connection()
        .await
        .map_err(|e| js_sys::Error::new(&e.to_string()).into())
}

/// Accepts on the ambient loop and moves each connection onto its own.
async fn accept_loop(listener: TcpListener) {
    loop {
        match listener.accept().await {
            Ok((stream, peer)) => {
                // Deregister from the ambient reactor; the connection's own
                // event loop registers it again on first use.
                let std = match stream.into_std() {
                    Ok(std) => std,
                    Err(e) => {
                        console_error!("{peer}: {e}");
                        continue;
                    }
                };
                // Create a new isolated event loop for the connection handler
                // This avoids cross-context IO between different incoming requests
                schedule_isolated(
                    async move {
                        let stream = TcpStream::from_std(std)?;
                        echo(stream).await
                    },
                    move |out| match out {
                        Ok(Err(e)) => console_error!("{peer}: {e}"),
                        Err(join) => console_error!("{peer}: connection task failed: {join}"),
                        Ok(Ok(())) => {}
                    },
                );
            }
            Err(e) => console_error!("accept failed: {e}"),
        }
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
