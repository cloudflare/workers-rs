// Regression test for https://github.com/cloudflare/workers-rs/issues/973.
// A `#[event(start)]` handler must compile without `wasm-bindgen` as a direct
// dependency. The macro should resolve `wasm_bindgen` through `::worker::`.
// It must also not conflict when `wasm_bindgen` is already in scope, or when
// multiple `#[event(start)]` handlers exist in the same module.
use worker::{event, wasm_bindgen};

#[event(start)]
pub fn setup_hook() {}

#[event(start)]
pub fn another_hook() {}

fn main() {}
