use worker::*;

/// A simple SQLite-backed counter stored in Durable Object storage.
///
/// Each Durable Object instance owns its own private SQLite database.  We keep a
/// single table `counter` with one row that stores the current value.
#[durable_object]
pub struct SqlCounter {
    sql: SqlStorage,
}

#[durable_object]
impl DurableObject for SqlCounter {
    fn new(state: State, _env: Env) -> Self {
        let sql = state.storage().sql();
        // Create table if it does not exist.  Note: `exec` is synchronous.
        sql.exec("CREATE TABLE IF NOT EXISTS counter(value INTEGER);", None)
            .expect("create table");
        Self { sql }
    }

    async fn fetch(&self, _req: Request) -> Result<Response> {
        // Read current value (if any)
        #[derive(serde::Deserialize)]
        struct Row {
            value: i32,
        }

        let rows: Vec<Row> = self
            .sql
            .exec("SELECT value FROM counter LIMIT 1;", None)?
            .to_array()?;
        let current = rows.first().map(|r| r.value).unwrap_or(0);
        let next = current + 1;

        // Upsert new value – simplest way: delete and insert again.
        self.sql.exec("DELETE FROM counter;", None)?;
        self.sql
            .exec("INSERT INTO counter(value) VALUES (?);", vec![next.into()])?;

        Response::ok(format!("SQL counter is now {}", next))
    }
}

#[worker::send]
/// Route handler that proxies a request to our SqlCounter Durable Object with id derived from the
/// path `/sql-counter/{name}` (so every name gets its own instance).
pub async fn handle_sql_counter(
    req: Request,
    env: Env,
    _data: super::SomeSharedData,
) -> Result<Response> {
    let uri = req.url()?;
    let mut segments = uri.path_segments().unwrap();
    // skip "sql-counter"
    let _ = segments.next();
    let name = segments.next().unwrap_or("default");
    let namespace = env.durable_object("SQL_COUNTER")?;
    let stub = namespace.id_from_name(name)?.get_stub()?;
    stub.fetch_with_str("https://fake-host/").await
}
