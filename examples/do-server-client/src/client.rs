use worker::*;

#[event(fetch)]
async fn main(_req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let namespace = env.durable_object("DO_SERVER")?;
    let stub = namespace.id_from_name("A")?.get_stub()?;

    stub.fetch_with_str("http://fake_url.com/3+4").await
}
