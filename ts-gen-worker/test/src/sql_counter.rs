use crate::Env;
use ts_gen_worker::*;

use crate::sql::{self, SqlStorageValue};

/// A simple SQLite-backed counter stored in Durable Object storage.
#[durable_object]
pub struct SqlCounter {
    sql: SqlStorage,
}

impl DurableObject for SqlCounter {
    type Env = Env;

    fn new(state: DurableObjectState, _env: Env) -> SqlCounter {
        let sql = state.storage().sql();
        sql::exec(
            &sql,
            "CREATE TABLE IF NOT EXISTS counter(value INTEGER);",
            None,
        )
        .expect("create table");
        Self { sql }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let url = req.url_parsed()?;
        let path = url.path();

        if path.contains("/set-large/") {
            self.handle_set_large_value(&url)
        } else {
            self.handle_increment()
        }
    }
}

impl SqlCounter {
    fn handle_increment(&self) -> Result<Response> {
        #[derive(serde::Deserialize)]
        struct Row {
            value: i32,
        }

        let rows: Vec<Row> = sql::to_array(&sql::exec(
            &self.sql,
            "SELECT value FROM counter LIMIT 1;",
            None,
        )?)?;
        let current = rows.first().map_or(0, |r| r.value);
        let next = current + 1;

        sql::exec(&self.sql, "DELETE FROM counter;", None)?;
        sql::exec(
            &self.sql,
            "INSERT INTO counter(value) VALUES (?);",
            vec![next.into()],
        )?;

        Response::builder().ok(format!("SQL counter is now {next}"))
    }

    fn handle_set_large_value(&self, url: &url::Url) -> Result<Response> {
        let path = url.path();
        let value_str = path.split("/set-large/").nth(1).unwrap_or("0");

        let large_value: i64 = value_str
            .parse()
            .map_err(|_| Error::from("Invalid number format"))?;

        let safe_value = match SqlStorageValue::try_from_i64(large_value) {
            Ok(value) => value,
            Err(e) => {
                return Response::builder().ok(format!(
                    "Error: Cannot store value {large_value} - {e}. JavaScript safe range is ±9007199254740991"
                ));
            }
        };

        sql::exec(&self.sql, "DELETE FROM counter;", None)?;
        sql::exec(
            &self.sql,
            "INSERT INTO counter(value) VALUES (?);",
            vec![safe_value],
        )?;

        Response::builder().ok(format!("Successfully stored large value: {large_value}"))
    }
}

/// Route handler that proxies a request to our SqlCounter Durable Object with id derived from the
/// path `/sql-counter/{name}` (so every name gets its own instance).
pub async fn handle_sql_counter(
    req: Request,
    env: Env,
    _data: super::SomeSharedData,
) -> Result<Response> {
    let uri = req.url_parsed()?;
    let mut segments = uri.path_segments().unwrap();
    let _ = segments.next();
    let name = segments.next().unwrap_or("default");

    let remaining_path: Vec<&str> = segments.collect();
    let full_path = if remaining_path.is_empty() {
        "https://fake-host/".to_string()
    } else {
        format!("https://fake-host/{}", remaining_path.join("/"))
    };

    let namespace = env.sql_counter();
    let stub = namespace.get(&namespace.id_from_name(name)?)?;
    stub.fetch_with_str(&full_path).await
}
