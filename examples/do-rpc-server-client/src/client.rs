#![allow(clippy::empty_docs)]

use worker::*;

pub mod rpc {
    include!(concat!(env!("OUT_DIR"), "/calculator.rs"));
}

#[event(fetch)]
async fn main(_req: Request, env: Env, _ctx: Context) -> Result<Response> {
    use rpc::Calculator;

    let namespace = env.durable_object("DO_RPC_SERVER")?;
    let stub = namespace.id_from_name("A")?.get_stub()?;

    //stub.fetch_with_str("http://fake_url.com/3+4").await

    //let service: rpc::CalculatorService = env.durable_object("DO_RPC_SERVER")?.into();

    //let num = service.add(1, 2).await?;

    //Response::ok(format!("Hello World {:?}", num))

    let service: rpc::CalculatorService = stub.into();

    let left = 8;
    let right = 8;

    let result = service.add(left, right).await?;
    Response::ok(&format!("{} + {} = {}", left, right, result))
}
