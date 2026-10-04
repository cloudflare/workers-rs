use worker::*;

/// This handler is invoked for every inbound UDP flow to your Worker.
#[event(connect(udp))]
async fn connect(mut socket: UdpSocket, _env: Env, _ctx: Context) -> Result<()> {
    while let Some(datagram) = socket.recv().await? {
        socket.send(&datagram).await?;
    }
    Ok(())
}
