use worker::postgres_tls::PassthroughTls;
use worker::*;

#[event(fetch)]
async fn main(_req: Request, env: Env, _ctx: Context) -> anyhow::Result<Response> {
    let hyperdrive = env.hyperdrive("DB")?;

    // Connect using Worker Socket
    let socket = Socket::builder()
        .secure_transport(SecureTransport::StartTls)
        .connect(hyperdrive.host(), hyperdrive.port())?;

    let config = hyperdrive
        .connection_string()
        .parse::<tokio_postgres::Config>()?;

    let (client, connection) = config.connect_raw(socket, PassthroughTls).await?;

    wasm_bindgen_futures::spawn_local(async move {
        if let Err(error) = connection.await {
            console_log!("connection error: {:?}", error);
        }
    });

    // Setup table:
    // CREATE TABLE IF NOT EXISTS foo (id SERIAL PRIMARY KEY, name TEXT);
    // INSERT INTO foo (name) VALUES ('Fred');
    let result = client.query("SELECT * FROM FOO", &[]).await?;

    for row in result {
        let id: i32 = row.get(0);
        let name: &str = row.get(1);
        console_log!("id: {}, name: {}", id, name);
    }

    Ok(Response::ok("Hello, World!")?)
}
