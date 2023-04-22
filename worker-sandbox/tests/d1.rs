use worker_sandbox::{CreatePerson, Person};

use crate::util::{expect_wrangler, get, post, delete};

mod util;

fn drop_table() {
    let response = post("d1/exec", |rb| rb.body("drop table if exists people;"));

    assert!(response.status().is_success());
}

fn create_table() {
    let query = "CREATE TABLE IF NOT EXISTS people ( \
            id INTEGER PRIMARY KEY, \
            name TEXT NOT NULL, \
            age INTEGER NOT NULL)";
    let response = post("d1/exec", |rb| rb.body(query));
    assert!(response.status().is_success());
}

fn setup() {
    drop_table();
    create_table();
}

#[test]
fn d1_create_table() {
    expect_wrangler();
    setup();
}

#[test]
fn d1_insert_prepare_first() {
    expect_wrangler();
    setup();
    let person = create_person();

    assert_eq!("Ada LoveLace", person.name);
    assert_eq!(32, person.age);
}

#[test]
fn d1_select_prepare_all() {
    expect_wrangler();
    setup();
    let person = create_person();

    let response = get("d1/people", |rb| rb);
    let people: Vec<Person> = serde_json::from_str(response.text().unwrap().as_str()).unwrap();

    assert!(people.contains(&person));
}

#[test]
fn d1_select_prepare_first() {
    expect_wrangler();
    setup();
    let person = create_person();

    let response = get(format!("d1/people/{}", person.id).as_str(), |rb| rb);
    let person_by_id: Person = serde_json::from_str(response.text().unwrap().as_str()).unwrap();

    assert_eq!(person_by_id, person);
}

#[test]
fn d1_delete_prepare_run() {
    expect_wrangler();
    setup();
    let person = create_person();
    let response = delete(format!("d1/people/{}", person.id).as_str(), |rb| rb);

    if response.status().is_success() {
        let response = get("d1/people", |rb| rb);
        let people: Vec<Person> = serde_json::from_str(response.text().unwrap().as_str()).unwrap();
        assert!(!people.contains(&person));
    } else {
        panic!("failed to delete person");
    }
}

#[test]
fn d1_update_prepare() {
    expect_wrangler();
    setup();
    let og_person = create_person();

    let update_person = Person {
        id: og_person.id,
        age: 100,
        name: "Ada L".to_string()
    };

    post(format!("d1/people/{}", update_person.id).as_str(), |rb| rb.body(serde_json::to_string(&update_person).unwrap()));

    let response = get("d1/people", |rb| rb);
    let people: Vec<Person> = serde_json::from_str(response.text().unwrap().as_str()).unwrap();
    assert!(people.contains(&update_person));
    assert!(!people.contains(&og_person));
}

fn create_person() -> Person {
    let new_person = CreatePerson {
        name: "Ada LoveLace".to_string(),
        age: 32,
    };
    let response = post("d1/people", |rb| rb.body(serde_json::to_string(&new_person).unwrap()));
    let person: Person = serde_json::from_str(response.text().unwrap().as_str()).unwrap();
    person
}




