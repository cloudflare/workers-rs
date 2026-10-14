use wasm_bindgen::prelude::*;

/// A trap (panic under `panic=abort`, `unreachable`, OOM, a JS exception crossing
/// Wasm frames) leaves the shadow stack and Rust state inconsistent, so the glue
/// reinstantiates the module before the next call into it. Requires
/// `wasm-bindgen --force-enable-abort-handler`.
fn on_abort() {
    wasm_bindgen::handler::schedule_reinit();
}

#[wasm_bindgen(start, private)]
fn init() {
    wasm_bindgen::handler::set_on_abort(on_abort);
}
