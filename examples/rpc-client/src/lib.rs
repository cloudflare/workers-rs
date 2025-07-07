#![allow(clippy::empty_docs)]

use worker::*;

pub mod rpc {
    include!(concat!(env!("OUT_DIR"), "/calculator.rs"));
}

#[event(fetch)]
async fn main(_req: Request, env: Env, _ctx: Context) -> Result<Response> {
    use rpc::Calculator;

    let service: rpc::CalculatorService = env.service("SERVER")?.into();

    let num = service.add(1, 2).await?;

    Response::ok(format!("{num:?}"))
}
