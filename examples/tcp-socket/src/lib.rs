use tokio::io::{AsyncReadExt, AsyncWriteExt};
use worker::*;

/// This handler is invoked for every inbound TCP connection to your Worker.
/// The `socket` argument implements `tokio::io::AsyncRead` + `AsyncWrite`,
/// so you can use the familiar tokio APIs to read and write data.
#[event(connect)]
async fn connect(mut socket: Socket, _env: Env, _ctx: Context) -> Result<()> {
    console_log!("New inbound TCP connection");

    // Read up to 512 bytes from the client
    let mut buf = vec![0u8; 512];
    match socket.read(&mut buf).await {
        Ok(n) if n > 0 => {
            console_log!("Received {} bytes", n);

            // Echo the data back to the client
            socket.write_all(&buf[..n]).await?;
            socket.flush().await?;

            console_log!("Echoed {} bytes back to client", n);
        }
        Ok(_) => {
            console_log!("Client closed connection immediately");
        }
        Err(e) => {
            console_error!("Error reading from socket: {}", e);
        }
    }

    // The socket is closed automatically when this handler returns
    Ok(())
}
