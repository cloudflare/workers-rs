use serde::Serialize;
use serde_json::json;
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use worker::{Ai, Env, Request, Response, Result};

use crate::SomeSharedData;

#[wasm_bindgen(inline_js = "export function mock_ai() {
  class Ai { async run(model, inputs) { return JSON.stringify(inputs); } }
  return new Ai();
}")]
extern "C" {
    fn mock_ai() -> JsValue;
}

#[derive(Serialize)]
struct Input {
    prompt: &'static str,
    max_tokens: Option<u32>,
    tool: serde_json::Value,
}

pub async fn handle_ai_run_input(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let ai: Ai = mock_ai().unchecked_into();
    let wire: String = ai
        .run(
            "@cf/test",
            Input {
                prompt: "hi",
                max_tokens: None,
                tool: json!({"parameters": {"type": "object", "properties": {}}}),
            },
        )
        .await?;
    Response::ok(wire)
}
