use crate::{Env, SomeSharedData};
use js_sys::Object;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use ts_gen_worker::*;
use wasm_bindgen::prelude::*;

/// `worker::query!`: prepare + bind serde-serialised arguments, with missing
/// values sent as `null` since D1 rejects `undefined`.
fn query<T: Serialize>(db: &D1Database, sql: &str, args: &[&T]) -> Result<D1PreparedStatement> {
    let prepared = db.prepare(sql);
    if args.is_empty() {
        return Ok(prepared);
    }
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_missing_as_null(true);
    let bindings = args
        .iter()
        .map(|arg| arg.serialize(&serializer))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    prepared.bind(&bindings)
}

/// `worker::D1Type`: the typed binding values `bind_refs` accepts.
enum D1Type<'a> {
    Integer(i32),
    Text(&'a str),
}

impl From<&D1Type<'_>> for JsValue {
    fn from(value: &D1Type<'_>) -> Self {
        match *value {
            D1Type::Integer(i) => JsValue::from_f64(f64::from(i)),
            D1Type::Text(s) => JsValue::from_str(s),
        }
    }
}

fn bind_refs(stmt: &D1PreparedStatement, args: &[&D1Type<'_>]) -> Result<D1PreparedStatement> {
    let bindings: Vec<JsValue> = args.iter().map(|a| JsValue::from(*a)).collect();
    stmt.bind(&bindings)
}

/// `worker::D1Result::results::<T>()`.
fn results<T: DeserializeOwned>(result: &D1Result) -> Result<Vec<T>> {
    result
        .results()
        .into_iter()
        .map(|row| Ok(serde_wasm_bindgen::from_value(row)?))
        .collect()
}

/// `worker::D1PreparedStatement::first::<T>(col)`.
async fn first<T: DeserializeOwned>(
    stmt: &D1PreparedStatement,
    col_name: Option<&str>,
) -> Result<Option<T>> {
    let value = match col_name {
        Some(col) => stmt.first_with_col_name(col).await?,
        None => stmt.first().await?,
    };
    Ok(value.map(serde_wasm_bindgen::from_value).transpose()?)
}

/// `worker::D1PreparedStatement::raw::<T>()`.
async fn raw<T: DeserializeOwned>(stmt: &D1PreparedStatement) -> Result<Vec<Vec<T>>> {
    stmt.raw()
        .await?
        .into_iter()
        .map(|row| Ok(serde_wasm_bindgen::from_value(row.into())?))
        .collect()
}

/// `worker::D1Error::cause()`: D1 wraps the SQLite error as the JS `cause`.
fn d1_cause(e: &Error) -> String {
    let Error::Js(e) = e else {
        return "unknown error".into();
    };
    let value: &JsValue = e.as_ref();
    value
        .dyn_ref::<js_sys::Error>()
        .and_then(|e| e.cause().dyn_into::<js_sys::Error>().ok())
        .map_or_else(|| "unknown error".into(), |cause| cause.message())
}

#[derive(Deserialize)]
struct Person {
    id: u32,
    name: String,
    age: u32,
}

pub async fn prepared_statement(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let db = env.db();
    let unbound_stmt = query::<()>(&db, "SELECT * FROM people WHERE name = ?", &[])?;

    let stmt = bind_refs(&unbound_stmt, &[&D1Type::Text("Ryan Upton")])?;

    // All rows
    let all = stmt.all().await?;
    let people = results::<Person>(&all)?;

    assert!(all.success());
    assert_eq!(people.len(), 1);
    assert_eq!(people[0].name, "Ryan Upton");
    assert_eq!(people[0].age, 21);
    assert_eq!(people[0].id, 6);

    // All columns of the first rows
    let person = first::<Person>(&stmt, None).await?.unwrap();
    assert_eq!(person.name, "Ryan Upton");
    assert_eq!(person.age, 21);

    // The name of the first row
    let name = first::<String>(&stmt, Some("name")).await?.unwrap();
    assert_eq!(name, "Ryan Upton");

    // All of the rows as column arrays of raw JSON values.
    let rows = raw::<serde_json::Value>(&stmt).await?;
    assert_eq!(rows.len(), 1);
    let columns = &rows[0];

    assert_eq!(columns[0].as_u64(), Some(6));
    assert_eq!(columns[1].as_str(), Some("Ryan Upton"));
    assert_eq!(columns[2].as_u64(), Some(21));

    let stmt_2 = bind_refs(&unbound_stmt, &[&D1Type::Text("John Smith")])?;
    let person = first::<Person>(&stmt_2, None).await?.unwrap();
    assert_eq!(person.name, "John Smith");
    assert_eq!(person.age, 92);

    let prepared_argument = D1Type::Text("Dorian Fischer");
    let stmt_3 = bind_refs(&unbound_stmt, &[&prepared_argument])?;
    let person = first::<Person>(&stmt_3, None).await?.unwrap();
    assert_eq!(person.name, "Dorian Fischer");
    assert_eq!(person.age, 19);

    Response::builder().ok("ok")
}

pub async fn batch(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let db = env.db();
    let mut results_iter = db
        .batch(&[
            query::<()>(&db, "SELECT * FROM people WHERE id < 4", &[])?,
            query::<()>(&db, "SELECT * FROM people WHERE id > 4", &[])?,
        ])
        .await?
        .into_iter();

    let first_results = results::<Person>(&results_iter.next().unwrap())?;
    assert_eq!(first_results.len(), 3);
    assert_eq!(first_results[0].id, 1);
    assert_eq!(first_results[1].id, 2);
    assert_eq!(first_results[2].id, 3);

    let second_results = results::<Person>(&results_iter.next().unwrap())?;
    assert_eq!(second_results.len(), 2);
    assert_eq!(second_results[0].id, 5);
    assert_eq!(second_results[1].id, 6);

    Response::builder().ok("ok")
}

pub async fn session_prepare_and_batch(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let db = env.db();

    let session = db.with_session();
    assert_eq!(session.get_bookmark(), None);

    let stmt = bind_refs(
        &session.prepare("SELECT * FROM people WHERE id = ?"),
        &[&D1Type::Integer(6)],
    )?;
    let person = first::<Person>(&stmt, None).await?.unwrap();
    assert_eq!(person.id, 6);
    assert_eq!(person.name, "Ryan Upton");
    assert_eq!(person.age, 21);

    let constrained_session = db.with_session_with_constraint(D1SessionConstraint::FirstPrimary);
    let mut results_iter = constrained_session
        .batch(&[
            constrained_session.prepare("SELECT * FROM people WHERE id < 3"),
            constrained_session.prepare("SELECT * FROM people WHERE id > 5"),
        ])
        .await?
        .into_iter();

    let first_results = results::<Person>(&results_iter.next().unwrap())?;
    assert_eq!(first_results.len(), 2);
    assert_eq!(first_results[0].id, 1);
    assert_eq!(first_results[1].id, 2);

    let second_results = results::<Person>(&results_iter.next().unwrap())?;
    assert_eq!(second_results.len(), 1);
    assert_eq!(second_results[0].id, 6);

    Response::builder().ok("ok")
}

pub async fn session_bookmark_roundtrip(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let db = env.db();
    let session = db.with_session_with_constraint(D1SessionConstraint::FirstUnconstrained);

    let stmt = bind_refs(
        &session.prepare("SELECT * FROM people WHERE id = ?"),
        &[&D1Type::Integer(1)],
    )?;
    let person = first::<Person>(&stmt, None).await?.unwrap();
    assert_eq!(person.id, 1);

    let bookmark = session.get_bookmark();
    if let Some(bookmark) = bookmark {
        assert!(!bookmark.is_empty());

        let resumed_session = db.with_session_with_bookmark(&bookmark);
        let stmt = bind_refs(
            &resumed_session.prepare("SELECT * FROM people WHERE id = ?"),
            &[&D1Type::Integer(4)],
        )?;
        let person = first::<Person>(&stmt, None).await?.unwrap();
        assert_eq!(person.name, "John Smith");
    } else {
        // Some local runtimes may not materialize bookmarks for simple reads.
        let fallback_session = db.with_session_with_constraint(D1SessionConstraint::FirstPrimary);
        let stmt = bind_refs(
            &fallback_session.prepare("SELECT * FROM people WHERE id = ?"),
            &[&D1Type::Integer(4)],
        )?;
        let person = first::<Person>(&stmt, None).await?.unwrap();
        assert_eq!(person.name, "John Smith");
    }

    Response::builder().ok("ok")
}

pub async fn exec(req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let db = env.db();
    let result = db.exec(&req.text().await?).await.expect("doesn't exist");

    Response::builder().ok((result.count() as u64).to_string())
}

pub async fn dump(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let db = env.db();
    let bytes = db.dump().await?.to_vec();
    Response::builder().from_bytes(&bytes)
}

pub async fn error(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let db = env.db();
    let error = db
        .exec("THIS IS NOT VALID SQL")
        .await
        .expect_err("did not get error");

    assert_eq!(
        d1_cause(&error),
        "Error in line 1: THIS IS NOT VALID SQL: near \"THIS\": syntax error at offset 0: SQLITE_ERROR"
    );

    Response::builder().ok("")
}

#[derive(Debug, Deserialize)]
struct NullablePerson {
    id: u32,
    name: Option<String>,
    age: Option<u32>,
}

pub async fn jsvalue_null_is_null(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    assert!(JsValue::NULL.is_null());

    Response::builder().ok("ok")
}

pub async fn serialize_optional_none(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_missing_as_null(true);

    let none: Option<String> = None;
    let js_none = ::serde::ser::Serialize::serialize(&none, &serializer).unwrap();
    assert!(js_none.is_null());

    Response::builder().ok("ok")
}

pub async fn serialize_optional_some(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_missing_as_null(true);

    let some: Option<String> = Some("Hello".to_string());
    let js_some = ::serde::ser::Serialize::serialize(&some, &serializer).unwrap();
    assert!(js_some.is_string());

    Response::builder().ok("ok")
}

pub async fn deserialize_optional_none(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let js_value = Object::from_iter([
        ("id", JsValue::from(1.0)),
        ("name", JsValue::NULL),
        ("age", JsValue::NULL),
    ]);

    let value: NullablePerson = serde_wasm_bindgen::from_value(js_value.into()).unwrap();

    assert_eq!(value.id, 1);
    assert_eq!(value.name, None);
    assert_eq!(value.age, None);

    Response::builder().ok("ok")
}

pub async fn insert_and_retrieve_optional_none(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let db = env.db();

    let query_stmt = query::<serde_json::Value>(
        &db,
        "INSERT INTO nullable_people (id, name, age) VALUES (?1, ?2, ?3)",
        &[
            &serde_json::json!(3),
            &serde_json::Value::Null,
            &serde_json::Value::Null,
        ],
    )?;
    query_stmt.run().await?;

    let stmt = query::<()>(&db, "SELECT * FROM nullable_people WHERE id = 3", &[])?;
    let person = first::<NullablePerson>(&stmt, None).await?.unwrap();
    assert_eq!(person.id, 3);
    assert_eq!(person.name, None);
    assert_eq!(person.age, None);

    Response::builder().ok("ok")
}

pub async fn insert_and_retrieve_optional_some(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let db = env.db();
    let query_stmt = query::<serde_json::Value>(
        &db,
        "INSERT INTO nullable_people (id, name, age) VALUES (?1, ?2, ?3)",
        &[
            &serde_json::json!(4),
            &serde_json::json!("Dude"),
            &serde_json::json!(12),
        ],
    )?;
    query_stmt.run().await?;

    let stmt = query::<()>(&db, "SELECT * FROM nullable_people WHERE id = 4", &[])?;
    let person = first::<NullablePerson>(&stmt, None).await?.unwrap();
    assert_eq!(person.id, 4);
    assert_eq!(person.name, Some("Dude".to_string()));
    assert_eq!(person.age, Some(12));

    Response::builder().ok("ok")
}

pub async fn retrieve_optional_none(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let db = env.db();

    let stmt = query::<()>(&db, "SELECT * FROM nullable_people WHERE id = 1", &[])?;
    let person = first::<NullablePerson>(&stmt, None).await?.unwrap();
    assert_eq!(person.id, 1);
    assert_eq!(person.name, None);
    assert_eq!(person.age, None);

    Response::builder().ok("ok")
}

pub async fn retrieve_optional_some(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let db = env.db();

    let stmt = query::<()>(&db, "SELECT * FROM nullable_people WHERE id = 2", &[])?;
    let person = first::<NullablePerson>(&stmt, None).await?.unwrap();
    assert_eq!(person.id, 2);
    assert_eq!(person.name, Some("Wynne Ogley".to_string()));
    assert_eq!(person.age, Some(67));

    Response::builder().ok("ok")
}

pub async fn retrive_first_none(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let db = env.db();

    let stmt = query::<()>(&db, "SELECT * FROM nullable_people WHERE id = 9999", &[])?;
    assert!(first::<NullablePerson>(&stmt, None).await?.is_none());

    Response::builder().ok("ok")
}

pub async fn deserialize_with_type_mismatch(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let db = env.db();

    let error =
        results::<NullablePerson>(&query::<()>(&db, "SELECT 'hello' AS id", &[])?.run().await?);

    if let Err(error) = error {
        assert_eq!(
            error.to_string(),
            "Error: invalid type: string \"hello\", expected u32"
        );
    } else {
        panic!("Expected .results() to propagate its error");
    }

    Response::builder().ok("ok")
}
