use serde_json::{json, Value};
use worker::*;

#[durable_object]
pub struct Test {
    state: State,
}

impl DurableObject for Test {
    fn new(state: State, _env: Env) -> Self {
        Self { state }
    }

    async fn fetch(&self, _req: Request) -> Result<Response> {
        let kv = self.state.storage().kv();

        // CLEAN
        kv.delete("a");
        kv.delete("b");

        // CREATE
        kv.put("a", json!({ "x": 1 }))?;
        kv.put("b", json!({ "x": 2 }))?;

        // READ
        let a: Option<Value> = kv.get("a")?;
        let b: Option<Value> = kv.get("b")?;
        let missing: Option<Value> = kv.get("c")?;

        // UPDATE
        kv.put("a", json!({ "x": 42 }))?;
        let a_updated: Option<Value> = kv.get("a")?;

        // DELETE
        let deleted = kv.delete("b");
        let after_delete: Option<Value> = kv.get("b")?;

        // LIST
        let mut list = Vec::new();
        for item in kv.list::<Value>()? {
            let (k, v) = item?;
            list.push((k, v));
        }

        Response::from_json(&json!({
            "read": { "a": a, "b": b, "missing": missing },
            "update": a_updated,
            "delete": { "deleted": deleted, "after_delete": after_delete },
            "list": list
        }))
    }
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let durable_obj = env.durable_object("TEST")?;
    let stub = durable_obj.id_from_name("A")?.get_stub()?;
    stub.fetch_with_request(req).await
}
