use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
struct Foo {
    id: String
    msg: String
}
