use crate::util::{block_concurrency_while, sleep};
use crate::Env;
use js_sys::Uint8Array;
use serde::Serialize;
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use ts_gen_worker::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[durable_object]
pub struct MyClass {
    state: DurableObjectState,
    number: Cell<usize>,
    ctor_name: Option<String>,
    init_runs: Rc<Cell<usize>>,
    limit: Rc<Cell<u64>>,
}

impl DurableObject for MyClass {
    type Env = Env;

    fn new(state: DurableObjectState, _env: Env) -> MyClass {
        let ctor_name = state.id().name();
        let init_runs = Rc::new(Cell::new(0));
        let limit = Rc::new(Cell::new(0));
        let (runs, l, storage) = (init_runs.clone(), limit.clone(), state.storage());
        let init = block_concurrency_while(&state, async move {
            runs.set(runs.get() + 1);
            sleep(200.0).await;
            let stored: u64 = storage.get_into("limit").await?.unwrap_or(100);
            storage.put_serialized("init_probe", &stored).await?;
            l.set(stored);
            Ok(())
        });
        spawn_local(async move {
            let _ = init.await;
        });
        Self {
            state,
            number: Cell::new(0),
            ctor_name,
            init_runs,
            limit,
        }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let handler = async move {
            match req.path().as_str() {
                "/hello" => {
                    let name = self
                        .state
                        .id()
                        .name()
                        .unwrap_or_else(|| "unknown".to_string());
                    Response::builder().ok(format!("Hello from {name}!"))
                }
                "/ctor-name" => {
                    let name = self
                        .ctor_name
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string());
                    Response::builder().ok(format!("Hello from {name}!"))
                }
                "/storage" => {
                    let storage = self.state.storage();
                    let map = [("one".to_string(), 1), ("two".to_string(), 2)]
                        .iter()
                        .cloned()
                        .collect::<HashMap<_, _>>();
                    storage.put_serialized("map", &map).await?;
                    storage
                        .put_serialized("array", &[("one", 1), ("two", 2)])
                        .await?;
                    storage.put_serialized("anything", &Some(45)).await?;

                    let list = storage.list().await?;
                    let mut keys = vec![];

                    for key in list.keys() {
                        if key != "count" {
                            keys.push(key);
                        }
                    }

                    assert_eq!(
                        keys,
                        vec!["anything", "array", "map"],
                        "Didn't list all of the keys: {keys:?}"
                    );
                    let vals = storage
                        .get_with_keys(&keys)
                        .await
                        .map_err(|e| Error::from(format!("{e:?} -- get_multiple")))?;
                    let val = |key| vals.get(key).unwrap_or_default();
                    assert_eq!(
                        serde_wasm_bindgen::from_value::<Option<i32>>(val("anything"))?,
                        Some(45),
                        "Didn't get the right Option<i32> using get_multiple"
                    );
                    assert_eq!(
                        serde_wasm_bindgen::from_value::<[(String, i32); 2]>(val("array"))?,
                        [("one".to_string(), 1), ("two".to_string(), 2)],
                        "Didn't get the right array using get_multiple"
                    );
                    assert_eq!(
                        serde_wasm_bindgen::from_value::<HashMap<String, i32>>(val("map"))?,
                        map,
                        "Didn't get the right HashMap<String, i32> using get_multiple"
                    );

                    {
                        let bytes = Uint8Array::new_with_length(3);
                        bytes.copy_from(b"123");
                        storage.put("bytes", &bytes).await?;
                        let bytes: Vec<u8> = storage
                            .get_into("bytes")
                            .await?
                            .expect("get after put yielded nothing");
                        storage.delete("bytes").await?;
                        assert_eq!(
                            bytes, b"123",
                            "eficient serialization of bytes is not preserved"
                        );
                    }

                    #[derive(Serialize)]
                    struct Stuff {
                        thing: String,
                        other: i32,
                    }
                    storage
                        .put_entries_serialized(&Stuff {
                            thing: "Hello there".to_string(),
                            other: 56,
                        })
                        .await?;

                    assert_eq!(
                        storage
                            .get_into::<String>("thing")
                            .await?
                            .expect("get('thing') yielded nothing"),
                        "Hello there",
                        "Didn't put the right thing with put_multiple"
                    );
                    assert_eq!(
                        storage
                            .get_into::<i32>("other")
                            .await?
                            .expect("get('other') yielded nothing"),
                        56,
                        "Didn't put the right thing with put_multiple"
                    );

                    storage.delete_with_keys(&["thing", "other"]).await?;

                    {
                        const BAR: &[u8] = b"bar";
                        let entries = js_sys::Object::from_iter([("foo", Uint8Array::from(BAR))]);
                        storage.put_with_entries(&entries).await?;
                        assert_eq!(
                            storage
                                .get_into::<Vec<u8>>("foo")
                                .await?
                                .expect("get('foo') yielded nothing"),
                            BAR,
                            "Didn't the right thing with put_multiple_raw"
                        );
                    }

                    self.number
                        .set(storage.get_into("count").await?.unwrap_or(0) + 1);

                    storage.delete_all().await?;

                    let count = self.number.get();
                    storage.put_serialized("count", &count).await?;
                    Response::builder().ok(self.number.get().to_string())
                }
                "/transaction" => {
                    Response::builder().error("transactional storage API is still unstable", 501)
                }
                "/limit" => Response::builder().ok(format!(
                    "limit:{}:{}",
                    self.limit.get(),
                    self.init_runs.get()
                )),
                "/block-concurrency" => {
                    let storage = self.state.storage();
                    let count: usize = block_concurrency_while(&self.state, async move {
                        let current: usize = storage.get_into("bcw_count").await?.unwrap_or(0);
                        let next = current + 1;
                        storage.put_serialized("bcw_count", &next).await?;
                        Ok(next)
                    })
                    .await?;
                    Response::builder().ok(count.to_string())
                }
                "/block-concurrency-errors-as-values" => {
                    self.number.set(self.number.get() + 1);
                    let calls = self.number.get();

                    let storage = self.state.storage();
                    let outcome: std::result::Result<String, String> =
                        block_concurrency_while(&self.state, async move {
                            let _seen: Option<usize> =
                                storage.get_into("bcw_count").await.ok().flatten();
                            Ok(Err("simulated transient failure".to_string()))
                        })
                        .await?;

                    match outcome {
                        Ok(value) => Response::builder().ok(format!("ok:{value}:{calls}")),
                        Err(err) => Response::builder().ok(format!("err:{err}:{calls}")),
                    }
                }
                "/block-concurrency-reset-count" => {
                    self.number.set(self.number.get() + 1);
                    Response::builder().ok(self.number.get().to_string())
                }
                "/block-concurrency-reset-trigger" => {
                    block_concurrency_while(&self.state, async move {
                        Err::<(), _>("intentional reset".into())
                    })
                    .await?;
                    Response::builder().ok("unreachable")
                }
                _ => Response::builder().error("Not Found", 404),
            }
        };
        handler
            .await
            .or_else(|e: Error| Response::builder().error(e.to_string(), 500))
    }
}

#[durable_object]
pub struct AnotherClass {
    #[allow(unused)]
    state: DurableObjectState,
    counter: Cell<i32>,
}

impl DurableObject for AnotherClass {
    type Env = Env;

    fn new(state: DurableObjectState, _env: Env) -> AnotherClass {
        Self {
            state,
            counter: Cell::new(0),
        }
    }

    async fn fetch(&self, _req: Request) -> Result<Response> {
        self.counter.set(self.counter.get() + 1);
        Response::builder().ok(format!("Counter: {}", self.counter.get()))
    }
}

pub async fn handle_hello(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.my_class();
    let name = "my-durable-object";
    let id = namespace.id_from_name(name)?;
    let stub = namespace.get(&id)?;
    stub.fetch_with_str("https://fake-host/hello").await
}

pub async fn handle_ctor_name(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.my_class();
    let name = "my-durable-object";
    let id = namespace.id_from_name(name)?;
    let stub = namespace.get(&id)?;
    stub.fetch_with_str("https://fake-host/ctor-name").await
}

pub async fn handle_hello_unique(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.my_class();
    let id = namespace.new_unique_id()?;
    let stub = namespace.get(&id)?;
    stub.fetch_with_str("https://fake-host/hello").await
}

pub async fn handle_storage(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.my_class();
    let stub = namespace.get(&namespace.id_from_name("singleton")?)?;
    stub.fetch_with_str("https://fake-host/storage").await
}

pub async fn handle_basic_test(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace: DurableObjectNamespace = env.my_class();
    let id = namespace.id_from_name("A")?;
    assert_eq!(id.name().as_deref(), Some("A"), "Missing name");
    assert!(
        namespace.new_unique_id()?.name().is_none(),
        "Expected name property to be absent"
    );
    let bad = env.get::<JsValue>("DFSDF_FAKE_BINDING");
    assert!(bad.is_err(), "Invalid binding did not raise error");

    let stub = namespace.get(&id)?;
    let res = stub
        .fetch_with_str("https://fake-host/hello")
        .await?
        .text()
        .await?;
    let res2 = stub
        .fetch(&Request::new_with_str_and_init(
            "https://fake-host/hello",
            &RequestInit::builder()
                .body_with_str("lol")
                .method("POST")
                .build(),
        )?)
        .await?
        .text()
        .await?;

    assert_eq!(res, res2, "Durable object responded wrong to 'hello'");

    let res = stub
        .fetch_with_str("https://fake-host/storage")
        .await?
        .text()
        .await?;
    let num = res.parse::<usize>().map_err(|_| {
        Error::from("Durable Object responded wrong to 'storage': ".to_string() + &res)
    })?;
    let res = stub
        .fetch_with_str("https://fake-host/storage")
        .await?
        .text()
        .await?;
    let num2 = res
        .parse::<usize>()
        .map_err(|_| Error::from("Durable Object responded wrong to 'storage'"))?;

    assert_eq!(num2, num + 1, "Durable object responded wrong to 'storage'");

    let res = stub
        .fetch_with_str("https://fake-host/transaction")
        .await?
        .text()
        .await?;
    let num = res.parse::<usize>().map_err(|_| {
        Error::from("Durable Object responded wrong to 'transaction': ".to_string() + &res)
    })?;

    assert_eq!(
        num,
        num2 + 1,
        "Durable object responded wrong to 'transaction'"
    );

    Response::builder().ok("ok")
}

pub async fn handle_block_concurrency(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.my_class();
    let stub = namespace.get(&namespace.id_from_name("block-concurrency")?)?;
    stub.fetch_with_str("https://fake-host/block-concurrency")
        .await
}

pub async fn handle_block_concurrency_reset_count(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.my_class();
    let stub = namespace.get(&namespace.id_from_name("block-concurrency-reset")?)?;
    stub.fetch_with_str("https://fake-host/block-concurrency-reset-count")
        .await
}

pub async fn handle_block_concurrency_reset_trigger(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.my_class();
    let stub = namespace.get(&namespace.id_from_name("block-concurrency-reset")?)?;
    stub.fetch_with_str("https://fake-host/block-concurrency-reset-trigger")
        .await
}

pub async fn handle_constructor_init(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.my_class();
    let stub = namespace.get(&namespace.id_from_name("constructor-init")?)?;
    stub.fetch_with_str("https://fake-host/limit").await
}

pub async fn handle_block_concurrency_errors_as_values(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.my_class();
    let stub = namespace.get(&namespace.id_from_name("block-concurrency-errors-as-values")?)?;
    stub.fetch_with_str("https://fake-host/block-concurrency-errors-as-values")
        .await
}

pub async fn handle_get_by_name(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.my_class();
    let name = "my-durable-object";

    let stub = namespace.get_by_name(name)?;

    stub.fetch_with_str("https://fake-host/hello").await
}

pub async fn handle_get_by_name_with_location_hint(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.my_class();
    let name = "my-durable-object";

    let stub = namespace.get_by_name_with_options(
        name,
        &DurableObjectNamespaceGetDurableObjectOptions::builder()
            .location_hint(DurableObjectLocationHint::Enam)
            .build(),
    )?;

    stub.fetch_with_str("https://fake-host/hello").await
}
