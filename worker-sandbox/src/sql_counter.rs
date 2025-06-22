#[allow(clippy::wildcard_imports)]
use worker::*;

/// A simple SQLite-backed counter stored in Durable Object storage.
///
/// Each Durable Object instance owns its own private SQLite database.  We keep a
/// single table `counter` with one row that stores the current value.
#[durable_object]
pub struct SqlCounter {
    sql: SqlStorage,
}

impl DurableObject for SqlCounter {
    fn new(state: State, _env: Env) -> Self {
        let sql = state.storage().sql();
        // Create table if it does not exist.  Note: `exec` is synchronous.
        sql.exec("CREATE TABLE IF NOT EXISTS counter(value INTEGER);", None)
            .expect("create table");
        Self { sql }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let url = req.url()?;
        let path = url.path();

        // Parse path to determine action
        if path.contains("/set-large/") {
            self.handle_set_large_value(&url)
        } else {
            self.handle_increment()
        }
    }
}

impl SqlCounter {
    fn handle_increment(&self) -> Result<Response> {
        // Read current value (if any)
        #[derive(serde::Deserialize)]
        struct Row {
            value: i32,
        }

        let rows: Vec<Row> = self
            .sql
            .exec("SELECT value FROM counter LIMIT 1;", None)?
            .to_array()?;
        let current = rows.first().map_or(0, |r| r.value);
        let next = current + 1;

        // Upsert new value – simplest way: delete and insert again.
        self.sql.exec("DELETE FROM counter;", None)?;
        self.sql
            .exec("INSERT INTO counter(value) VALUES (?);", vec![next.into()])?;

        Response::ok(format!("SQL counter is now {next}"))
    }

    fn handle_set_large_value(&self, url: &worker::Url) -> Result<Response> {
        // Extract the large value from the path /set-large/{value}
        let path = url.path();
        let value_str = path.split("/set-large/").nth(1).unwrap_or("0");

        // Parse the value as i64
        let large_value: i64 = value_str
            .parse()
            .map_err(|_| worker::Error::from("Invalid number format"))?;

        // Use try_from_i64 to safely create SqlStorageValue
        let safe_value = match worker::SqlStorageValue::try_from_i64(large_value) {
            Ok(value) => value,
            Err(e) => {
                return Response::ok(format!(
                    "Error: Cannot store value {large_value} - {e}. JavaScript safe range is ±9007199254740991"
                ));
            }
        };

        // Store the safe value
        self.sql.exec("DELETE FROM counter;", None)?;
        self.sql
            .exec("INSERT INTO counter(value) VALUES (?);", vec![safe_value])?;

        Response::ok(format!("Successfully stored large value: {large_value}"))
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

    // Build the remaining path for the durable object request
    let remaining_path: Vec<&str> = segments.collect();
    let full_path = if remaining_path.is_empty() {
        "https://fake-host/".to_string()
    } else {
        format!("https://fake-host/{}", remaining_path.join("/"))
    };

    let namespace = env.durable_object("SQL_COUNTER")?;
    let stub = namespace.id_from_name(name)?.get_stub()?;
    stub.fetch_with_str(&full_path).await
}
