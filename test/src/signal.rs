use worker::signal;
use worker::{Env, Request, Response, Result};

use crate::SomeSharedData;

pub async fn handle_signal_poll(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let is_registered = signal::is_registered();
    let is_near_limit = signal::is_near_cpu_limit();
    Response::ok(format!("{is_registered}:{is_near_limit}"))
}
