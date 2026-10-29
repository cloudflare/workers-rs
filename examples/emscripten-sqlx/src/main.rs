use std::str::FromStr;

use sqlx::{Connection, Row, postgres::PgConnectOptions};
use worker::*;

fn main() {}

#[event(fetch)]
async fn fetch(_req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let hyperdrive = env.hyperdrive("HYPERDRIVE")?;
    let connection_string = hyperdrive.connection_string();
    let opts = PgConnectOptions::from_str(&connection_string).map_err(|e| Error::RustError(e.to_string()))?;
    let mut conn = sqlx::PgConnection::connect_with(&opts).await.map_err(|e| Error::RustError(e.to_string()))?;

    let row = sqlx::query("SELECT uuidv7()::text").fetch_one(&mut conn).await.map_err(|e| Error::RustError(format!("select uuidv7() failed: {e}")))?;
    let uuid: String = row.get(0);
    Response::ok(uuid)
}
