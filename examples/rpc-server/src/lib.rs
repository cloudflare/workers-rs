use wasm_bindgen::prelude::wasm_bindgen;
use worker::*;

#[event(fetch)]
async fn main(_req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    Response::ok("Hello World")
}

#[wasm_bindgen]
pub async fn add(a: Vec<u32>) -> u32 {
    a.iter().sum()
}
