use serde::Serialize;
use serde_json::json;
use worker::{js_sys, Request, Response, Result};

use crate::SomeSharedData;
use worker::Env;

#[worker::send]
pub async fn test_json_compatible_serde_value(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let val = json!({"parameters": {"type": "object", "properties": {}}});

    let default_ser = serde_wasm_bindgen::to_value(&val).unwrap();
    let default_name = js_sys::Object::constructor(&default_ser.into())
        .name()
        .as_string()
        .unwrap_or_default();

    let compat_ser = val
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .unwrap();
    let compat_name = js_sys::Object::constructor(&compat_ser.into())
        .name()
        .as_string()
        .unwrap_or_default();

    Response::from_json(&json!({
        "default": default_name,
        "json_compatible": compat_name,
    }))
}
