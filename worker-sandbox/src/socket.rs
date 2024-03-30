use crate::SomeSharedData;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use worker::{ConnectionBuilder, Env, Error, Request, Response, Result};

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
        Err(e) => Response::ok(format!("{:?}", e)),
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
