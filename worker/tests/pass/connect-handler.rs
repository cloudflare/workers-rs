// A correctly-shaped `#[event(connect)]` handler must compile.
use worker::{event, Context, Env, Result, Socket};

#[event(connect)]
async fn connect(_socket: Socket, _env: Env, _ctx: Context) -> Result<()> {
    Ok(())
}

fn main() {}
