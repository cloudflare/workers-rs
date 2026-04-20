use worker::*;

#[durable_object]
pub struct SynchronousStorage {
    state: State,
}

impl DurableObject for SynchronousStorage {
    fn new(state: State, _env: Env) -> Self {
        Self { state }
    }

    async fn fetch(&self, _req: Request) -> Result<Response> {
        let sync_kv = self.state.storage().kv();

        let first = serde_json::json!({"x": 1});
        let second = serde_json::json!({"x": 2});

        sync_kv.put("first", first.clone())?;
        sync_kv.put("second", second.clone())?;

        assert_eq!(sync_kv.get("first")?, Some(first.clone()));
        assert_eq!(sync_kv.get("second")?, Some(second.clone()));

        let mut original = [
            (String::from("first"), first),
            (String::from("second"), second),
        ];
        let mut list: Box<[(String, serde_json::Value)]> =
            sync_kv.list().filter_map(Result::ok).collect();

        original.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        list.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        assert_eq!(original.as_slice(), list.as_ref());

        assert!(sync_kv.delete("first"));
        assert!(sync_kv.delete("second"));

        Ok(Response::empty()?.with_status(204))
    }
}

#[worker::send]
pub async fn handle_synchronous_storage(
    req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.durable_object("SYNCHRONOUS_STORAGE")?;
    let stub = namespace.unique_id()?.get_stub()?;
    stub.fetch_with_request(req).await
}