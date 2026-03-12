use worker::signal::Signal;
use worker::{Env, Request, Response, Result};

use crate::SomeSharedData;

pub async fn handle_signal_poll(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let signal = Signal::poll();
    let is_listening = signal.is_listening();
    let value = signal.value();
    Response::ok(format!("{is_listening}:{value}"))
}
