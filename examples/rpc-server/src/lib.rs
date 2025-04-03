use worker::*;
use wasm_bindgen::prelude::wasm_bindgen;


#[event(fetch)]
async fn main(_req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    Response::ok("Hello World")
}

#[rpc]
impl Rpc {

    #[rpc]
    pub async fn add(&self, a: u32, b: u32) -> u32 {
        console_error_panic_hook::set_once();
        a + b
    }
}
