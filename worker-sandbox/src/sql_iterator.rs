use serde::Deserialize;
use worker::{durable_object, wasm_bindgen, Env, Request, Response, Result, SqlStorage, State};

/// A Durable Object that demonstrates SQL cursor iterator methods.
///
/// This example creates a table with sample data and provides endpoints
/// to test both the next() and raw() iterator methods.
#[durable_object]
pub struct SqlIterator {
    sql: SqlStorage,
}

#[derive(Deserialize)]
struct Product {
    id: i32,
    name: String,
    price: f64,
    in_stock: i32,
}

#[derive(Deserialize)]
struct BadProduct {
    id: String, // This will cause deserialization to fail since id is actually an integer
    name: String,
    price: f64,
    in_stock: i32,
}

impl DurableObject for SqlIterator {
    fn new(state: State, _env: Env) -> Self {
        let sql = state.storage().sql();

        // Create table and seed with test data
        sql.exec(
            "CREATE TABLE IF NOT EXISTS products(id INTEGER PRIMARY KEY, name TEXT, price REAL, in_stock INTEGER);",
            None,
        ).expect("create table");

        // Check if we need to seed data
        let count: Vec<serde_json::Value> = sql
            .exec("SELECT COUNT(*) as count FROM products;", None)
            .expect("count query")
            .to_array()
            .expect("count result");

        if count
            .first()
            .and_then(|v| v.get("count"))
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0)
            == 0
        {
            // Seed with test data
            let products = vec![
                ("Laptop", 999.99, true),
                ("Mouse", 29.99, true),
                ("Keyboard", 79.99, false),
                ("Monitor", 299.99, true),
                ("Headphones", 149.99, false),
            ];

            for (name, price, in_stock) in products {
                sql.exec(
                    "INSERT INTO products(name, price, in_stock) VALUES (?, ?, ?);",
                    vec![name.into(), price.into(), i32::from(in_stock).into()],
                )
                .expect("insert product");
            }
        }

        Self { sql }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let url = req.url()?;
        let path = url.path();

        match path {
            "/next" => self.handle_next(),
            "/raw" => self.handle_raw(),
            "/next-invalid" => self.handle_next_invalid(),
            _ => Response::ok("SQL Iterator Test - try /next, /raw, or /next-invalid endpoints"),
        }
    }
}

impl SqlIterator {
    fn handle_next(&self) -> Result<Response> {
        let cursor = self.sql.exec("SELECT * FROM products ORDER BY id;", None)?;

        let mut results = Vec::new();
        let iterator = cursor.next::<Product>();

        for result in iterator {
            match result {
                Ok(product) => {
                    results.push(format!(
                        "Product {}: {} - ${:.2} (in stock: {})",
                        product.id,
                        product.name,
                        product.price,
                        product.in_stock != 0
                    ));
                }
                Err(e) => {
                    results.push(format!("Error deserializing row: {e}"));
                }
            }
        }

        let response_body = format!("next() iterator results:\n{}", results.join("\n"));

        Response::ok(response_body)
    }

    fn handle_raw(&self) -> Result<Response> {
        let cursor = self.sql.exec("SELECT * FROM products ORDER BY id;", None)?;

        let mut results = Vec::new();
        let column_names = cursor.column_names();
        results.push(format!("Columns: {}", column_names.join(", ")));

        let iterator = cursor.raw();

        for result in iterator {
            match result {
                Ok(row) => {
                    let row_str: Vec<String> = row.iter().map(|v| format!("{v:?}")).collect();
                    results.push(format!("Row: [{}]", row_str.join(", ")));
                }
                Err(e) => {
                    results.push(format!("Error reading row: {e}"));
                }
            }
        }

        let response_body = format!("raw() iterator results:\n{}", results.join("\n"));

        Response::ok(response_body)
    }

    fn handle_next_invalid(&self) -> Result<Response> {
        let cursor = self.sql.exec("SELECT * FROM products ORDER BY id;", None)?;

        let mut results = Vec::new();
        let iterator = cursor.next::<BadProduct>();

        for result in iterator {
            match result {
                Ok(product) => {
                    results.push(format!(
                        "BadProduct {}: {} - ${:.2} (in stock: {})",
                        product.id,
                        product.name,
                        product.price,
                        product.in_stock != 0
                    ));
                }
                Err(e) => {
                    results.push(format!("Error deserializing row: {e}"));
                }
            }
        }

        let response_body = format!("next-invalid() iterator results:\n{}", results.join("\n"));

        Response::ok(response_body)
    }
}

/// Route handler for the SQL iterator test Durable Object.
pub async fn handle_sql_iterator(
    req: Request,
    env: Env,
    _data: super::SomeSharedData,
) -> Result<Response> {
    let uri = req.url()?;
    let mut segments = uri.path_segments().unwrap();
    // skip "sql-iterator"
    let _ = segments.next();

    // Get name and remaining path
    let name = segments.next().unwrap_or("default");
    let remaining_path: Vec<&str> = segments.collect();
    let path = if remaining_path.is_empty() {
        "/"
    } else {
        &format!("/{}", remaining_path.join("/"))
    };

    let namespace = env.durable_object("SQL_ITERATOR")?;
    let stub = namespace.id_from_name(name)?.get_stub()?;

    // Forward the request path to the DO
    let new_url = format!("https://fake-host{path}");

    stub.fetch_with_str(&new_url).await
}
