use worker::postgres_tls::PassthroughTls;
use worker::*;

#[event(fetch)]
async fn main(_req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let mut config = tokio_postgres::config::Config::new();
    // Configure username, password, database as needed.
    config.user("postgres");

    // Connect using Worker Socket
    let socket = Socket::builder()
        .secure_transport(SecureTransport::StartTls)
        .connect("database_url", 5432)?;
    let (_client, connection) = config
        .connect_raw(socket, PassthroughTls)
        .await
        .map_err(|e| worker::Error::RustError(format!("tokio-postgres: {:?}", e)))?;

    wasm_bindgen_futures::spawn_local(async move {
        if let Err(error) = connection.await {
            console_log!("connection error: {:?}", error);
        }
    });

    // Use `client` to make queries.

    Ok(Response::new("Hello, World!".into()))
}
