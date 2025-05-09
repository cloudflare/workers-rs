use wasm_bindgen::prelude::wasm_bindgen;
use worker::*;

#[durable_object]
pub struct RPCServer {
    state: State,
    counter: u32,
}

#[wasm_bindgen]
impl RPCServer {
    #[wasm_bindgen]
    pub fn add(&mut self, a: u32, b: u32) -> u32 {
        console_error_panic_hook::set_once();
        self.counter += 1;
        a + b + self.counter
    }
}

#[durable_object]
impl DurableObject for RPCServer {
    fn new(state: State, _env: Env) -> Self {
        Self { state, counter: 0 }
    }
}
