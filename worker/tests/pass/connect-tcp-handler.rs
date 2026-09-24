use worker::{event, Context, Env, Result, Socket};

#[event(connect(tcp))]
async fn connect(_socket: Socket, _env: Env, _ctx: Context) -> Result<()> {
    Ok(())
}

fn main() {}
