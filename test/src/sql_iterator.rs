use serde::Deserialize;
use worker::{
    DurableObject, Env, Request, Response, Result, SqlStorage, SqlStorageValue, State, durable_object, wasm_bindgen
};

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

#[derive(Debug)]
struct BlobData {
    id: i32,
    name: String,
    data: Vec<u8>,
}

impl BlobData {
    fn from_raw_row(row: &[SqlStorageValue]) -> Option<Self> {
        if row.len() != 3 {
            return None;
        }

        let id = match &row[0] {
            SqlStorageValue::Integer(i) => *i as i32,
            _ => return None,
        };

        let name = match &row[1] {
            SqlStorageValue::String(s) => s.clone(),
            _ => return None,
        };

        let data = match &row[2] {
            SqlStorageValue::Blob(bytes) => bytes.clone(),
            _ => return None,
        };

        Some(BlobData { id, name, data })
    }
}

impl DurableObject for SqlIterator {
    fn new(state: State, _env: Env) -> Self {
        let sql = state.storage().sql();

        // Create table and seed with test data
        sql.exec(
            "CREATE TABLE IF NOT EXISTS products(id INTEGER PRIMARY KEY, name TEXT, price REAL, in_stock INTEGER);",
            None,
        ).expect("create table");

        sql.exec(
            "CREATE TABLE IF NOT EXISTS blob_data(id INTEGER PRIMARY KEY, name TEXT, data BLOB);",
            None,
        )
        .expect("create blob table");

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

        let blob_count: Vec<serde_json::Value> = sql
            .exec("SELECT COUNT(*) as count FROM blob_data;", None)
            .expect("blob count query")
            .to_array()
            .expect("blob count result");

        if blob_count
            .first()
            .and_then(|v| v.get("count"))
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0)
            == 0
        {
            let blob_test_data = vec![
                ("binary_data", vec![0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE]),
                ("empty_blob", vec![]),
                ("text_as_blob", "Hello, World!".as_bytes().to_vec()),
                (
                    "large_blob",
                    (0u8..=255).cycle().take(1000).collect::<Vec<u8>>(),
                ),
            ];

            for (name, data) in blob_test_data {
                sql.exec(
                    "INSERT INTO blob_data(name, data) VALUES (?, ?);",
                    vec![name.into(), SqlStorageValue::Blob(data)],
                )
                .expect("insert blob data");
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
            "/blob-next" => self.handle_blob_next(),
            "/blob-raw" => self.handle_blob_raw(),
            "/blob-roundtrip" => self.handle_blob_roundtrip(),
            _ => Response::ok("SQL Iterator Test - try /next, /raw, /next-invalid, /blob-next, /blob-raw, or /blob-roundtrip endpoints"),
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

    fn handle_blob_next(&self) -> Result<Response> {
        let cursor = self
            .sql
            .exec("SELECT * FROM blob_data ORDER BY id;", None)?;

        let mut results = Vec::new();
        let iterator = cursor.raw();

        for result in iterator {
            match result {
                Ok(row) => {
                    if let Some(blob_data) = BlobData::from_raw_row(&row) {
                        let data_preview = if blob_data.data.len() <= 10 {
                            format!("{:?}", blob_data.data)
                        } else {
                            format!(
                                "{:?}...[{} bytes total]",
                                &blob_data.data[..10],
                                blob_data.data.len()
                            )
                        };

                        results.push(format!(
                            "BlobData {}: {} - data: {}",
                            blob_data.id, blob_data.name, data_preview
                        ));
                    } else {
                        results.push("Error: Failed to parse blob row".to_string());
                    }
                }
                Err(e) => {
                    results.push(format!("Error reading blob row: {e}"));
                }
            }
        }

        let response_body = format!("blob-next() iterator results:\n{}", results.join("\n"));

        Response::ok(response_body)
    }

    fn handle_blob_raw(&self) -> Result<Response> {
        let cursor = self
            .sql
            .exec("SELECT * FROM blob_data ORDER BY id;", None)?;

        let mut results = Vec::new();
        let column_names = cursor.column_names();
        results.push(format!("Columns: {}", column_names.join(", ")));

        let iterator = cursor.raw();

        for result in iterator {
            match result {
                Ok(row) => {
                    let mut row_str = Vec::new();
                    for (i, value) in row.iter().enumerate() {
                        if i == 2 {
                            // 'data' column is index 2 (id=0, name=1, data=2)
                            match value {
                                SqlStorageValue::Blob(bytes) => {
                                    if bytes.len() <= 10 {
                                        row_str.push(format!("Blob({:?})", bytes));
                                    } else {
                                        row_str.push(format!(
                                            "Blob({:?}...[{} bytes])",
                                            &bytes[..10],
                                            bytes.len()
                                        ));
                                    }
                                }
                                _ => row_str.push(format!("{:?}", value)),
                            }
                        } else {
                            row_str.push(format!("{:?}", value));
                        }
                    }
                    results.push(format!("Row: [{}]", row_str.join(", ")));
                }
                Err(e) => {
                    results.push(format!("Error reading blob row: {e}"));
                }
            }
        }

        let response_body = format!("blob-raw() iterator results:\n{}", results.join("\n"));

        Response::ok(response_body)
    }

    fn handle_blob_roundtrip(&self) -> Result<Response> {
        // Test data roundtrip: insert a BLOB and immediately read it back
        let test_data = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0xFF];
        let test_name = "roundtrip_test";

        // Insert test data
        self.sql.exec(
            "INSERT INTO blob_data(name, data) VALUES (?, ?);",
            vec![test_name.into(), SqlStorageValue::Blob(test_data.clone())],
        )?;

        // Read it back using both methods (raw iterator approach for both)
        let cursor_next = self.sql.exec(
            "SELECT * FROM blob_data WHERE name = ? ORDER BY id DESC LIMIT 1;",
            vec![test_name.into()],
        )?;

        let cursor_raw = self.sql.exec(
            "SELECT * FROM blob_data WHERE name = ? ORDER BY id DESC LIMIT 1;",
            vec![test_name.into()],
        )?;

        let mut results = Vec::new();
        results.push(format!("Original data: {:?}", test_data));

        // Test "next()" style result by converting raw data to BlobData struct
        let next_raw_iterator = cursor_next.raw();
        for result in next_raw_iterator {
            match result {
                Ok(row) => {
                    if let Some(blob_data) = BlobData::from_raw_row(&row) {
                        let matches = blob_data.data == test_data;
                        results.push(format!(
                            "next() result: {:?}, matches_original: {}",
                            blob_data.data, matches
                        ));
                    } else {
                        results.push("next() error: Failed to parse blob row".to_string());
                    }
                }
                Err(e) => {
                    results.push(format!("next() error: {e}"));
                }
            }
        }

        // Test raw iterator
        let raw_iterator = cursor_raw.raw();
        for result in raw_iterator {
            match result {
                Ok(row) => {
                    if let Some(SqlStorageValue::Blob(data)) = row.get(2) {
                        let matches = data == &test_data;
                        results.push(format!(
                            "raw() result: {:?}, matches_original: {}",
                            data, matches
                        ));
                    } else {
                        results.push("raw() error: data column is not a blob".to_string());
                    }
                }
                Err(e) => {
                    results.push(format!("raw() error: {e}"));
                }
            }
        }

        // Clean up test data
        self.sql.exec(
            "DELETE FROM blob_data WHERE name = ?;",
            vec![test_name.into()],
        )?;

        let response_body = format!("blob-roundtrip test results:\n{}", results.join("\n"));

        Response::ok(response_body)
    }
}

#[worker::send]
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
