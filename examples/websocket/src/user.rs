use wasm_bindgen::prelude::wasm_bindgen;
use worker::{wasm_bindgen, WebSocket};

// A random generated color used for css style on the frontend
#[wasm_bindgen(
    inline_js = "export function random_color() { return `#${Math.floor(Math.random() * 16777215).toString(16)}`; }"
)]
extern "C" {
    fn random_color() -> String;
}

#[derive(Clone)]
pub struct User {
    pub info: UserInfo,
    pub session: WebSocket,
}

impl User {
    pub fn new(id: String, name: String, session: WebSocket) -> Self {
        Self {
            info: UserInfo::new(id, name),
            session,
        }
    }
}

#[derive(Clone)]
pub struct UserInfo {
    /// Unique color of the user's name
    pub color: String,
    /// User's unique id
    pub id: String,
    /// User's name
    pub name: String,
}

impl UserInfo {
    fn new(id: String, name: String) -> Self {
        Self {
            color: random_color(),
            id,
            name,
        }
    }
}
