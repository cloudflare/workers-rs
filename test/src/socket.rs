use crate::SomeSharedData;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use worker::{ConnectionBuilder, Context, Env, Error, Request, Response, Result, Socket};

/// Connections handled by this Wasm instance; resets if the instance is reinitialized.
static CONNECTIONS: AtomicUsize = AtomicUsize::new(0);

#[worker::event(connect)]
pub async fn handle_connect(mut socket: Socket, _env: Env, _ctx: Context) -> Result<()> {
    let count = CONNECTIONS.fetch_add(1, Ordering::Relaxed) + 1;
    let mut request = [0; 4];
    socket.read_exact(&mut request).await?;
    match &request {
        b"fail" => return Err(Error::RustError("connect handler failed".into())),
        b"stat" => socket.write_all(count.to_string().as_bytes()).await?,
        _ => socket.write_all(&request).await?,
    }
    socket.flush().await?;
    Ok(())
}

#[worker::send]
pub async fn handle_socket_failed(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let socket = ConnectionBuilder::new().connect("127.0.0.1", 25000)?;

    match socket.opened().await {
        Ok(_) => {
            return Err(Error::RustError(
                "Socket should have failed to open.".to_owned(),
            ))
        }
        Err(e) => Response::ok(format!("{e:?}")),
    }
}

#[worker::send]
pub async fn handle_socket_read(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let mut socket = ConnectionBuilder::new().connect("127.0.0.1", 8080)?;

    socket.opened().await?;

    {
        socket.write_all(b"ping").await?;
    }

    let mut response = [0; 4];
    {
        socket.read_exact(&mut response).await?;
    }

    assert_eq!(&response, b"ping");

    socket.close().await?;

    socket.closed().await?;

    Response::ok("success")
}
