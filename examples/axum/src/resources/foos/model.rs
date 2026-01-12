use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Foo {
    pub id: String,
    pub msg: String,
}
