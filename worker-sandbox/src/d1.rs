use crate::SomeSharedData;
use crate::{
    js_sys::{Object, Reflect},
    wasm_bindgen,
};
use serde::Deserialize;
use wasm_bindgen::JsValue;
use worker::{D1PreparedArgument, D1Type, Env, Error, Request, Response, Result};

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
    let db = env.d1("DB")?;
    let unbound_stmt = worker::query!(&db, "SELECT * FROM people WHERE name = ?");

    let stmt = unbound_stmt.bind_refs(&D1Type::Text("Ryan Upton"))?;

    // All rows
    let results = stmt.all().await?;
    let people = results.results::<Person>()?;

    assert!(results.success());
    assert_eq!(results.error(), None);
    assert_eq!(people.len(), 1);
    assert_eq!(people[0].name, "Ryan Upton");
    assert_eq!(people[0].age, 21);
    assert_eq!(people[0].id, 6);

    // All columns of the first rows
    let person = stmt.first::<Person>(None).await?.unwrap();
    assert_eq!(person.name, "Ryan Upton");
    assert_eq!(person.age, 21);

    // The name of the first row
    let name = stmt.first::<String>(Some("name")).await?.unwrap();
    assert_eq!(name, "Ryan Upton");

    // All of the rows as column arrays of raw JSON values.
    let rows = stmt.raw::<serde_json::Value>().await?;
    assert_eq!(rows.len(), 1);
    let columns = &rows[0];

    assert_eq!(columns[0].as_u64(), Some(6));
    assert_eq!(columns[1].as_str(), Some("Ryan Upton"));
    assert_eq!(columns[2].as_u64(), Some(21));

    let stmt_2 = unbound_stmt.bind_refs([&D1Type::Text("John Smith")])?;
    let person = stmt_2.first::<Person>(None).await?.unwrap();
    assert_eq!(person.name, "John Smith");
    assert_eq!(person.age, 92);

    let prepared_argument = D1PreparedArgument::new(&D1Type::Text("Dorian Fischer"));
    let stmt_3 = unbound_stmt.bind_refs(&prepared_argument)?;
    let person = stmt_3.first::<Person>(None).await?.unwrap();
    assert_eq!(person.name, "Dorian Fischer");
    assert_eq!(person.age, 19);

    Response::ok("ok")
}

pub async fn batch(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let db = env.d1("DB")?;
    let mut results = db
        .batch(vec![
            worker::query!(&db, "SELECT * FROM people WHERE id < 4"),
            worker::query!(&db, "SELECT * FROM people WHERE id > 4"),
        ])
        .await?
        .into_iter();

    let first_results = results.next().unwrap().results::<Person>()?;
    assert_eq!(first_results.len(), 3);
    assert_eq!(first_results[0].id, 1);
    assert_eq!(first_results[1].id, 2);
    assert_eq!(first_results[2].id, 3);

    let second_results = results.next().unwrap().results::<Person>()?;
    assert_eq!(second_results.len(), 2);
    assert_eq!(second_results[0].id, 5);
    assert_eq!(second_results[1].id, 6);

    Response::ok("ok")
}

pub async fn exec(mut req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let db = env.d1("DB")?;
    let result = db
        .exec(req.text().await?.as_ref())
        .await
        .expect("doesn't exist");

    Response::ok(result.count()?.unwrap_or_default().to_string())
}

pub async fn dump(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let db = env.d1("DB")?;
    let bytes = db.dump().await?;
    Response::from_bytes(bytes)
}

pub async fn error(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let db = env.d1("DB")?;
    let error = db
        .exec("THIS IS NOT VALID SQL")
        .await
        .expect_err("did not get error");

    if let Error::D1(error) = error {
        assert_eq!(
            error.cause(),
            "Error in line 1: THIS IS NOT VALID SQL: near \"THIS\": syntax error at offset 0: SQLITE_ERROR"
        );
    } else {
        panic!("expected D1 error");
    }

    Response::ok("")
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
    console_error_panic_hook::set_once();

    assert!(wasm_bindgen::JsValue::NULL.is_null());

    Response::ok("ok")
}

pub async fn serialize_optional_none(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    console_error_panic_hook::set_once();
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_missing_as_null(true);

    let none: Option<String> = None;
    let js_none = ::serde::ser::Serialize::serialize(&none, &serializer).unwrap();
    assert!(js_none.is_null());

    Response::ok("ok")
}

pub async fn serialize_optional_some(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    console_error_panic_hook::set_once();
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_missing_as_null(true);

    let some: Option<String> = Some("Hello".to_string());
    let js_some = ::serde::ser::Serialize::serialize(&some, &serializer).unwrap();
    assert!(js_some.is_string());

    Response::ok("ok")
}

pub async fn deserialize_optional_none(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    console_error_panic_hook::set_once();

    let js_value = Object::new();
    Reflect::set(&js_value, &JsValue::from_str("id"), &JsValue::from_f64(1.0)).unwrap();
    Reflect::set(&js_value, &JsValue::from_str("name"), &JsValue::NULL).unwrap();
    Reflect::set(&js_value, &JsValue::from_str("age"), &JsValue::NULL).unwrap();

    let js_value: JsValue = js_value.into();

    let value: NullablePerson = serde_wasm_bindgen::from_value(js_value).unwrap();

    assert_eq!(value.id, 1);
    assert_eq!(value.name, None);
    assert_eq!(value.age, None);

    Response::ok("ok")
}

pub async fn insert_and_retrieve_optional_none(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let db = env.d1("DB")?;

    let query = worker::query!(
        &db,
        "INSERT INTO nullable_people (id, name, age) VALUES (?1, ?2, ?3)",
        &3,
        &None::<String>,
        &None::<u32>
    )?;
    query.run().await?;

    let stmt = worker::query!(&db, "SELECT * FROM nullable_people WHERE id = 3");
    let person = stmt.first::<NullablePerson>(None).await?.unwrap();
    assert_eq!(person.id, 3);
    assert_eq!(person.name, None);
    assert_eq!(person.age, None);

    Response::ok("ok")
}

pub async fn insert_and_retrieve_optional_some(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let db = env.d1("DB")?;
    let query = worker::query!(
        &db,
        "INSERT INTO nullable_people (id, name, age) VALUES (?1, ?2, ?3)",
        &4,
        &"Dude",
        &12
    )?;
    query.run().await?;

    let stmt = worker::query!(&db, "SELECT * FROM nullable_people WHERE id = 4");
    let person = stmt.first::<NullablePerson>(None).await?.unwrap();
    assert_eq!(person.id, 4);
    assert_eq!(person.name, Some("Dude".to_string()));
    assert_eq!(person.age, Some(12));

    Response::ok("ok")
}

pub async fn retrieve_optional_none(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let db = env.d1("DB")?;

    let stmt = worker::query!(&db, "SELECT * FROM nullable_people WHERE id = 1");
    let person = stmt.first::<NullablePerson>(None).await?.unwrap();
    assert_eq!(person.id, 1);
    assert_eq!(person.name, None);
    assert_eq!(person.age, None);

    Response::ok("ok")
}

pub async fn retrieve_optional_some(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let db = env.d1("DB")?;

    let stmt = worker::query!(&db, "SELECT * FROM nullable_people WHERE id = 2");
    let person = stmt.first::<NullablePerson>(None).await?.unwrap();
    assert_eq!(person.id, 2);
    assert_eq!(person.name, Some("Wynne Ogley".to_string()));
    assert_eq!(person.age, Some(67));

    Response::ok("ok")
}

pub async fn retrive_first_none(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let db = env.d1("DB")?;

    let stmt = worker::query!(&db, "SELECT * FROM nullable_people WHERE id = 9999");
    assert!(stmt.first::<NullablePerson>(None).await?.is_none());

    Response::ok("ok")
}
