use reqwest::blocking::Body;

use crate::util::{expect_wrangler, post_return_err};

mod util;

fn post_exec_and_expect<T: Into<Body>>(query: T, expected: u32) {
    let response = post_return_err(&format!("d1/exec"), |r| r.body(query.into())).unwrap();
    let status = response.status();
    let body = response.text().unwrap();
    if status.is_success() {
        let parsed = str::parse::<u32>(&body.clone());
        assert!(parsed.is_ok());
        assert_eq!(expected, parsed.unwrap());
    } else {
        let err = body.clone();
        eprintln!("Error received: {}", err);
        panic!("Error received from request.")
    }
}

// #[test]
// fn d1_exec_version() {
//     expect_wrangler();
//     post_exec_and_expect("PRAGMA schema_version", u32::MAX);
// }

#[test]
fn d1_create_table() {
    expect_wrangler();
    post_exec_and_expect(
        "CREATE TABLE IF NOT EXISTS people ( \
            person_id INTEGER PRIMARY KEY, \
            name TEXT NOT NULL, \
            age INTEGER NOT NULL)",
        1,
    );
}

#[test]
fn d1_insert_data() {
    expect_wrangler();
    d1_create_table();
    post_exec_and_expect(
        "INSERT OR IGNORE INTO people \
            (thing_id, title, description) \
            VALUES \
            (1, 'Freddie Pearce', 26), \
            (2, 'Wynne Ogley', 67), \
            (3, 'Dorian Fischer', 19), \
            (4, 'John Smith', 92), \
            (5, 'Magaret Willamson', 54), \
            (6, 'Ryan Upton', 21),",
        1,
    );
}
