use crate::util::{
    block_concurrency_while, error, ok, path, response_text, sleep, storage_get, storage_put,
};
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
            let stored: u64 = storage_get(&storage, "limit").await?.unwrap_or(100);
            storage_put(&storage, "init_probe", stored).await?;
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
            match path(&req)?.as_str() {
                "/hello" => {
                    let name = self
                        .state
                        .id()
                        .name()
                        .unwrap_or_else(|| "unknown".to_string());
                    ok(format!("Hello from {name}!"))
                }
                "/ctor-name" => {
                    let name = self
                        .ctor_name
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string());
                    ok(format!("Hello from {name}!"))
                }
                "/storage" => {
                    let storage = self.state.storage();
                    let map = [("one".to_string(), 1), ("two".to_string(), 2)]
                        .iter()
                        .cloned()
                        .collect::<HashMap<_, _>>();
                    storage_put(&storage, "map", map.clone()).await?;
                    storage_put(&storage, "array", [("one", 1), ("two", 2)]).await?;
                    storage_put(&storage, "anything", Some(45)).await?;

                    let list = storage.list().await?;
                    let mut keys = vec![];

                    for key in list.keys() {
                        let key = String::from(key?);
                        if key != "count" {
                            keys.push(key);
                        }
                    }

                    assert_eq!(
                        keys,
                        vec!["anything", "array", "map"],
                        "Didn't list all of the keys: {keys:?}"
                    );
                    // GAP: the `keys: string[]` overload's `Map` return collapsed into the
                    // first overload's `any`.
                    let vals: js_sys::Map = storage
                        .get_with_keys(&keys)
                        .await
                        .map_err(|e| Error::from(format!("{e:?} -- get_multiple")))?
                        .unchecked_into();
                    assert_eq!(
                        serde_wasm_bindgen::from_value::<Option<i32>>(vals.get(&"anything".into()))
                            .map_err(|e| e.to_string())?,
                        Some(45),
                        "Didn't get the right Option<i32> using get_multiple"
                    );
                    assert_eq!(
                        serde_wasm_bindgen::from_value::<[(String, i32); 2]>(
                            vals.get(&"array".into())
                        )
                        .map_err(|e| e.to_string())?,
                        [("one".to_string(), 1), ("two".to_string(), 2)],
                        "Didn't get the right array using get_multiple"
                    );
                    assert_eq!(
                        serde_wasm_bindgen::from_value::<HashMap<String, i32>>(
                            vals.get(&"map".into())
                        )
                        .map_err(|e| e.to_string())?,
                        map,
                        "Didn't get the right HashMap<String, i32> using get_multiple"
                    );

                    {
                        let bytes = Uint8Array::new_with_length(3);
                        bytes.copy_from(b"123");
                        storage.put("bytes", &bytes).await?;
                        let bytes: Vec<u8> = storage_get(&storage, "bytes")
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
                        .put_with_entries(
                            &serde_wasm_bindgen::to_value(&Stuff {
                                thing: "Hello there".to_string(),
                                other: 56,
                            })
                            .map_err(|e| e.to_string())?
                            .unchecked_into(),
                        )
                        .await?;

                    assert_eq!(
                        storage_get::<String>(&storage, "thing")
                            .await?
                            .expect("get('thing') yielded nothing"),
                        "Hello there",
                        "Didn't put the right thing with put_multiple"
                    );
                    assert_eq!(
                        storage_get::<i32>(&storage, "other")
                            .await?
                            .expect("get('other') yielded nothing"),
                        56,
                        "Didn't put the right thing with put_multiple"
                    );

                    storage
                        .delete_with_keys(&["thing".to_string(), "other".to_string()])
                        .await?;

                    {
                        const BAR: &[u8] = b"bar";
                        let obj = js_sys::Object::new();
                        let value = Uint8Array::new_with_length(u32::try_from(BAR.len()).unwrap());
                        value.copy_from(BAR);
                        js_sys::Reflect::set(&obj, &JsValue::from_str("foo"), &value.into())?;
                        storage.put_with_entries(&obj.unchecked_into()).await?;
                        assert_eq!(
                            storage_get::<Vec<u8>>(&storage, "foo")
                                .await?
                                .expect("get('foo') yielded nothing"),
                            BAR,
                            "Didn't the right thing with put_multiple_raw"
                        );
                    }

                    self.number
                        .set(storage_get(&storage, "count").await?.unwrap_or(0) + 1);

                    storage.delete_all().await?;

                    let count = self.number.get();
                    storage_put(&storage, "count", count).await?;
                    ok(self.number.get().to_string())
                }
                "/transaction" => error("transactional storage API is still unstable", 501),
                "/limit" => ok(format!(
                    "limit:{}:{}",
                    self.limit.get(),
                    self.init_runs.get()
                )),
                "/block-concurrency" => {
                    let storage = self.state.storage();
                    let count: usize = block_concurrency_while(&self.state, async move {
                        let current: usize = storage_get(&storage, "bcw_count").await?.unwrap_or(0);
                        let next = current + 1;
                        storage_put(&storage, "bcw_count", next).await?;
                        Ok(next)
                    })
                    .await?;
                    ok(count.to_string())
                }
                "/block-concurrency-errors-as-values" => {
                    self.number.set(self.number.get() + 1);
                    let calls = self.number.get();

                    let storage = self.state.storage();
                    let outcome: std::result::Result<String, String> =
                        block_concurrency_while(&self.state, async move {
                            let _seen: Option<usize> =
                                storage_get(&storage, "bcw_count").await.ok().flatten();
                            Ok(Err("simulated transient failure".to_string()))
                        })
                        .await?;

                    match outcome {
                        Ok(value) => ok(format!("ok:{value}:{calls}")),
                        Err(err) => ok(format!("err:{err}:{calls}")),
                    }
                }
                "/block-concurrency-reset-count" => {
                    self.number.set(self.number.get() + 1);
                    ok(self.number.get().to_string())
                }
                "/block-concurrency-reset-trigger" => {
                    block_concurrency_while(&self.state, async move {
                        Err::<(), _>("intentional reset".into())
                    })
                    .await?;
                    ok("unreachable")
                }
                _ => error("Not Found", 404),
            }
        };
        handler.await.or_else(|e: Error| error(e.to_string(), 500))
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
        ok(format!("Counter: {}", self.counter.get()))
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
    assert_eq!(id.name(), Some("A".into()), "Missing name");
    assert!(
        namespace.new_unique_id()?.name().is_none(),
        "Expected name property to be absent"
    );
    let bad = js_sys::Reflect::get(&env, &"DFSDF_FAKE_BINDING".into())?;
    assert!(bad.is_undefined(), "Invalid binding did not raise error");

    let stub = namespace.get(&id)?;
    let res = response_text(&stub.fetch_with_str("https://fake-host/hello").await?).await?;
    let res2 = response_text(
        &stub
            .fetch(&Request::new_with_str_and_init(
                "https://fake-host/hello",
                &RequestInit::builder()
                    .body_with_str("lol")
                    .method("POST")
                    .build(),
            )?)
            .await?,
    )
    .await?;

    assert_eq!(res, res2, "Durable object responded wrong to 'hello'");

    let res = response_text(&stub.fetch_with_str("https://fake-host/storage").await?).await?;
    let num = res.parse::<usize>().map_err(|_| {
        Error::from("Durable Object responded wrong to 'storage': ".to_string() + &res)
    })?;
    let res = response_text(&stub.fetch_with_str("https://fake-host/storage").await?).await?;
    let num2 = res
        .parse::<usize>()
        .map_err(|_| Error::from("Durable Object responded wrong to 'storage'"))?;

    assert_eq!(num2, num + 1, "Durable object responded wrong to 'storage'");

    let res = response_text(&stub.fetch_with_str("https://fake-host/transaction").await?).await?;
    let num = res.parse::<usize>().map_err(|_| {
        Error::from("Durable Object responded wrong to 'transaction': ".to_string() + &res)
    })?;

    assert_eq!(
        num,
        num2 + 1,
        "Durable object responded wrong to 'transaction'"
    );

    ok("ok")
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
