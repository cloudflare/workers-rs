use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

use wasm_bindgen_futures::JsFuture;
use worker::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type MyRpcInterface;

    #[wasm_bindgen(method, catch, js_name = "add")]
    pub fn add_raw(
        this: &MyRpcInterface,
        a: u32,
        b: u32,
    ) -> std::result::Result<js_sys::Promise, JsValue>;
}

impl MyRpcInterface {
    async fn add(&self, a: u32, b: u32) -> Result<u32> {
        let promise = self.add_raw(a, b)?;
        let fut = JsFuture::from(promise);
        let output = fut.await?;
        let num = output
            .as_f64()
            .ok_or_else(|| Error::JsError("output was not a number".to_owned()))?;
        Ok(num as u32)
    }
}

#[event(fetch)]
async fn main(_req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let service = env.service("SERVER")?;
    let rpc: MyRpcInterface = service.into_rpc();

    let num = rpc.add(1, 2).await?;

    Response::ok(format!("{:?}", num))
}
