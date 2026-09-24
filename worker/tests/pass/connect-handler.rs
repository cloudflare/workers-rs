// A correctly-shaped `#[event(connect)]` handler must compile.
use worker::{event, Context, Env, Result, Socket, UdpSocket};

#[event(connect)]
async fn connect(_socket: Socket, _env: Env, _ctx: Context) -> Result<()> {
    Ok(())
}

#[event(connect(udp))]
async fn connect_udp(_socket: UdpSocket, _env: Env, _ctx: Context) -> Result<()> {
    Ok(())
}

fn main() {}
