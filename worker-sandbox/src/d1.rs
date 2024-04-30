use crate::SomeSharedData;
use serde::Deserialize;
use worker::*;

#[derive(Deserialize)]
struct Person {
    id: u32,
    name: String,
    age: u32,
}

#[worker::send]
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

#[worker::send]
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

#[worker::send]
pub async fn exec(mut req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let db = env.d1("DB")?;
    let result = db
        .exec(req.text().await?.as_ref())
        .await
        .expect("doesn't exist");

    Response::ok(result.count()?.unwrap_or_default().to_string())
}

#[worker::send]
pub async fn dump(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let db = env.d1("DB")?;
    let bytes = db.dump().await?;
    Response::from_bytes(bytes)
}

#[worker::send]
pub async fn error(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let db = env.d1("DB")?;
    let error = db
        .exec("THIS IS NOT VALID SQL")
        .await
        .expect_err("did not get error");

    if let Error::D1(error) = error {
        assert_eq!(
            error.cause(),
            "Error in line 1: THIS IS NOT VALID SQL: near \"THIS\": syntax error at offset 0"
        )
    } else {
        panic!("expected D1 error");
    }

    Response::ok("")
}
