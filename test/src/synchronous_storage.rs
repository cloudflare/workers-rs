use worker::*;

#[durable_object]
pub struct SynchronousStorage {
    state: State,
}

impl DurableObject for SynchronousStorage {
    fn new(state: State, _env: Env) -> Self {
        Self { state }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        const KEYS_LEN: usize = 10;

        let sync_kv = self.state.storage().kv();
        let path = req.path();

        match path.as_str() {
            "/smoke" => {
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
                let mut list: Box<[_]> = sync_kv.list()?.map(Result::unwrap).collect();

                original.sort_unstable_by(|a, b| a.0.cmp(&b.0));
                list.sort_unstable_by(|a, b| a.0.cmp(&b.0));

                assert_eq!(original.as_slice(), list.as_ref());

                assert!(sync_kv.delete("first"));
                assert!(sync_kv.delete("second"));

                Response::ok("smoke ok")
            }
            "/overwrite" => {
                let overwrite = serde_json::json!({"v": 2});

                sync_kv.put("k", serde_json::json!({"v": 1}))?;
                sync_kv.put("k", overwrite.clone())?;

                assert_eq!(sync_kv.get("k")?, Some(overwrite));

                assert!(sync_kv.delete("k"));

                Response::ok("overwrite ok")
            }
            "/not_found" => {
                assert_eq!(sync_kv.get::<()>("nope")?, None);
                assert!(!sync_kv.delete("nope"));

                Response::ok("not_found ok")
            }
            "/list" => {
                let keys: [_; KEYS_LEN] = std::array::from_fn(|i| format!("k{i}"));

                for (i, key) in keys.iter().enumerate() {
                    sync_kv.put(key, serde_json::json!({ "i": i }))?;
                }

                let count = {
                    let list: SyncKvIterator<serde_json::Value> = sync_kv.list()?;
                    list.count()
                };

                assert_eq!(count, KEYS_LEN);

                for key in keys {
                    assert!(sync_kv.delete(&key));
                }

                Response::ok("list ok")
            }
            "/persist_fill" => {
                let keys: [_; KEYS_LEN] = std::array::from_fn(|i| format!("k{i}"));

                for (i, key) in keys.iter().enumerate() {
                    sync_kv.put(key, serde_json::json!({ "i": i }))?;
                }

                Response::ok("persist_fill ok")
            }
            "/persist_check" => {
                let keys: [_; KEYS_LEN] = std::array::from_fn(|i| format!("k{i}"));

                for (i, key) in keys.iter().enumerate() {
                    let val: Option<serde_json::Value> = sync_kv.get(key)?;

                    assert!(val.is_some());

                    assert_eq!(val, Some(serde_json::json!({"i": i})));
                }

                Response::ok("persist_check ok")
            }
            "/persist_cleanup" => {
                let list: SyncKvIterator<serde_json::Value> = sync_kv.list()?;

                let keys_collected: Box<[_]> =
                    list.filter_map(|e| e.ok().map(|(k, _)| k)).collect();

                for key in keys_collected {
                    assert!(sync_kv.delete(&key));
                }

                Response::ok("persist_cleanup ok")
            }
            _ => Response::error("unknown test", 404),
        }
    }
}

#[worker::send]
pub async fn handle_synchronous_storage_smoke(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.durable_object("SYNCHRONOUS_STORAGE")?;
    let stub = namespace.unique_id()?.get_stub()?;
    stub.fetch_with_str("http://fake-host/smoke").await
}

#[worker::send]
pub async fn handle_synchronous_storage_overwrite(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.durable_object("SYNCHRONOUS_STORAGE")?;
    let stub = namespace.unique_id()?.get_stub()?;
    stub.fetch_with_str("http://fake-host/overwrite").await
}

#[worker::send]
pub async fn handle_synchronous_storage_not_found(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.durable_object("SYNCHRONOUS_STORAGE")?;
    let stub = namespace.unique_id()?.get_stub()?;
    stub.fetch_with_str("http://fake-host/not_found").await
}

#[worker::send]
pub async fn handle_synchronous_storage_list(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.durable_object("SYNCHRONOUS_STORAGE")?;
    let stub = namespace.unique_id()?.get_stub()?;
    stub.fetch_with_str("http://fake-host/list").await
}

#[worker::send]
pub async fn handle_synchronous_storage_persist_fill(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.durable_object("SYNCHRONOUS_STORAGE")?;
    let stub = namespace.id_from_name("singleton")?.get_stub()?;
    stub.fetch_with_str("http://fake-host/persist_fill").await
}

#[worker::send]
pub async fn handle_synchronous_storage_persist_check(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.durable_object("SYNCHRONOUS_STORAGE")?;
    let stub = namespace.id_from_name("singleton")?.get_stub()?;
    stub.fetch_with_str("http://fake-host/persist_check").await
}

#[worker::send]
pub async fn handle_synchronous_storage_persist_cleanup(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.durable_object("SYNCHRONOUS_STORAGE")?;
    let stub = namespace.id_from_name("singleton")?.get_stub()?;
    stub.fetch_with_str("http://fake-host/persist_cleanup")
        .await
}