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

                assert!(sync_kv.delete("first")?);
                assert!(sync_kv.delete("second")?);

                Response::ok("smoke ok")
            }
            "/overwrite" => {
                let overwrite = serde_json::json!({"v": 2});

                sync_kv.put("k", serde_json::json!({"v": 1}))?;
                sync_kv.put("k", overwrite.clone())?;

                assert_eq!(sync_kv.get("k")?, Some(overwrite));

                assert!(sync_kv.delete("k")?);

                Response::ok("overwrite ok")
            }
            "/not_found" => {
                assert_eq!(sync_kv.get::<()>("nope")?, None);
                assert!(!sync_kv.delete("nope")?);

                Response::ok("not_found ok")
            }
            "/list" => {
                const KEYS_LEN: usize = 10;
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
                    assert!(sync_kv.delete(&key)?);
                }

                Response::ok("list ok")
            }
            "/list_options" => {
                // Seed: a, b, c, d, e
                for k in ["a", "b", "c", "d", "e"] {
                    sync_kv.put(k, serde_json::json!({"k": k}))?;
                }

                // start_after("b") yields c, d, e
                let after_b: Vec<String> = sync_kv
                    .list_with_options::<serde_json::Value>(
                        &SyncKvListOptionsBuilder::new().start_after("b").build(),
                    )?
                    .collect::<Result<Vec<_>>>()?
                    .into_iter()
                    .map(|(k, _)| k)
                    .collect();
                assert_eq!(after_b, vec!["c", "d", "e"]);

                // start("b") yields b, c, d, e
                let from_b: Vec<String> = sync_kv
                    .list_with_options::<serde_json::Value>(
                        &SyncKvListOptionsBuilder::new().start("b").build(),
                    )?
                    .collect::<Result<Vec<_>>>()?
                    .into_iter()
                    .map(|(k, _)| k)
                    .collect();
                assert_eq!(from_b, vec!["b", "c", "d", "e"]);

                // limit(2) + reverse(true) yields e, d
                let last_two: Vec<String> = sync_kv
                    .list_with_options::<serde_json::Value>(
                        &SyncKvListOptionsBuilder::new()
                            .reverse(true)
                            .limit(2)
                            .build(),
                    )?
                    .collect::<Result<Vec<_>>>()?
                    .into_iter()
                    .map(|(k, _)| k)
                    .collect();
                assert_eq!(last_two, vec!["e", "d"]);

                // end("c") yields a, b (exclusive)
                let until_c: Vec<String> = sync_kv
                    .list_with_options::<serde_json::Value>(
                        &SyncKvListOptionsBuilder::new().end("c").build(),
                    )?
                    .collect::<Result<Vec<_>>>()?
                    .into_iter()
                    .map(|(k, _)| k)
                    .collect();
                assert_eq!(until_c, vec!["a", "b"]);

                for k in ["a", "b", "c", "d", "e"] {
                    assert!(sync_kv.delete(k)?);
                }

                Response::ok("list_options ok")
            }
            "/persist_fill" => {
                const KEYS_LEN: usize = 10;
                let keys: [_; KEYS_LEN] = std::array::from_fn(|i| format!("k{i}"));

                for (i, key) in keys.iter().enumerate() {
                    sync_kv.put(key, serde_json::json!({ "i": i }))?;
                }

                Response::ok("persist_fill ok")
            }
            "/persist_check" => {
                const KEYS_LEN: usize = 10;
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
                    assert!(sync_kv.delete(&key)?);
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
pub async fn handle_synchronous_storage_list_options(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.durable_object("SYNCHRONOUS_STORAGE")?;
    let stub = namespace.unique_id()?.get_stub()?;
    stub.fetch_with_str("http://fake-host/list_options").await
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
