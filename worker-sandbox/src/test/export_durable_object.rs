use serde::Serialize;
use std::{cell::RefCell, collections::HashMap};

use worker::{
    durable_object, js_sys, js_sys::Uint8Array, wasm_bindgen::JsValue, Env, Request, Response,
    Result, State,
};

use crate::ensure;

#[durable_object]
pub struct MyClass {
    name: String,
    state: State,
    number: RefCell<usize>,
}

impl DurableObject for MyClass {
    fn new(state: State, env: Env) -> Self {
        // Check that we can re-derive the expected names.
        let namespace = env.durable_object("MY_CLASS").unwrap();
        let name = if let Some(name) = state.id().name() {
            assert!(state.id() == namespace.id_from_name(&name).unwrap());
            name
        } else {
            let id = state.id().to_string();
            assert!(state.id() == namespace.id_from_string(&id).unwrap());
            id
        };

        Self {
            name,
            state,
            number: RefCell::new(0),
        }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let handler = async move {
            match req.path().as_str() {
                "/hello" => Response::ok(format!("Hello from {}!", self.name)),
                "/storage" => {
                    let storage = self.state.storage();
                    let map = [("one".to_string(), 1), ("two".to_string(), 2)]
                        .iter()
                        .cloned()
                        .collect::<HashMap<_, _>>();
                    storage.put("map", map.clone()).await?;
                    storage.put("array", [("one", 1), ("two", 2)]).await?;
                    storage.put("anything", Some(45)).await?;

                    let list = storage.list().await?;
                    let mut keys = vec![];

                    for key in list.keys() {
                        let key = key?
                            .as_string()
                            .ok_or_else(|| "Key wasn't a string".to_string())?;
                        keys.push(key);
                    }

                    ensure!(
                        keys == vec!["anything", "array", "map"],
                        format!("Didn't list all of the keys: {keys:?}")
                    );
                    let vals = storage
                        .get_multiple(keys)
                        .await
                        .map_err(|e| e.to_string() + " -- get_multiple")?;
                    ensure!(
                        serde_wasm_bindgen::from_value::<Option<i32>>(
                            vals.get(&"anything".into())
                        )? == Some(45),
                        "Didn't get the right Option<i32> using get_multiple"
                    );
                    ensure!(
                        serde_wasm_bindgen::from_value::<[(String, i32); 2]>(
                            vals.get(&"array".into())
                        )? == [("one".to_string(), 1), ("two".to_string(), 2)],
                        "Didn't get the right array using get_multiple"
                    );
                    ensure!(
                        serde_wasm_bindgen::from_value::<HashMap<String, i32>>(
                            vals.get(&"map".into())
                        )? == map,
                        "Didn't get the right HashMap<String, i32> using get_multiple"
                    );

                    {
                        let bytes = Uint8Array::new_with_length(3);
                        bytes.copy_from(b"123");
                        storage.put_raw("bytes", bytes).await?;
                        let bytes = storage.get::<Vec<u8>>("bytes").await?;
                        storage.delete("bytes").await?;
                        ensure!(
                            bytes == b"123",
                            "eficient serialization of bytes is not preserved"
                        );
                    }

                    #[derive(Serialize)]
                    struct Stuff {
                        thing: String,
                        other: i32,
                    }
                    storage
                        .put_multiple(Stuff {
                            thing: "Hello there".to_string(),
                            other: 56,
                        })
                        .await?;

                    ensure!(
                        storage.get::<String>("thing").await? == "Hello there",
                        "Didn't put the right thing with put_multiple"
                    );
                    ensure!(
                        storage.get::<i32>("other").await? == 56,
                        "Didn't put the right thing with put_multiple"
                    );

                    storage.delete_multiple(vec!["thing", "other"]).await?;

                    {
                        let obj = js_sys::Object::new();
                        const BAR: &[u8] = b"bar";
                        let value = Uint8Array::new_with_length(BAR.len() as _);
                        value.copy_from(BAR);
                        js_sys::Reflect::set(&obj, &JsValue::from_str("foo"), &value.into())?;
                        storage.put_multiple_raw(obj).await?;
                        ensure!(
                            storage.get::<Vec<u8>>("foo").await? == BAR,
                            "Didn't the right thing with put_multiple_raw"
                        );
                    }

                    *self.number.borrow_mut() = storage.get("count").await.unwrap_or(0) + 1;

                    storage.delete_all().await?;

                    let count = *self.number.borrow();
                    storage.put("count", count).await?;
                    Response::ok(self.number.borrow().to_string())
                }
                "/transaction" => {
                    Response::error("transactional storage API is still unstable", 501)
                }
                _ => Response::error("Not Found", 404),
            }
        };
        handler
            .await
            .or_else(|err| Response::error(err.to_string(), 500))
    }
}

// Route handlers to exercise the Durable Object from tests.
#[worker::send]
pub async fn handle_hello(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let name = "your Durable Object";
    let namespace = env.durable_object("MY_CLASS")?;
    let id = namespace.id_from_name(name)?;
    // Same name gives the same ID
    assert!(id == namespace.id_from_name(name)?);

    // Same name but different namespaces gives different IDs
    let namespace2 = env.durable_object("COUNTER")?;
    assert!(id != namespace2.id_from_name(name)?);

    let stub = id.get_stub()?;
    stub.fetch_with_str("https://fake-host/hello").await
}

#[worker::send]
pub async fn handle_hello_unique(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.durable_object("MY_CLASS")?;
    let id = namespace.unique_id()?;
    // Different unique IDs should never be equal
    assert!(id != namespace.unique_id()?);
    // Deriving from the string form of the unique ID gives the same ID
    assert!(id == namespace.id_from_string(&id.to_string()).unwrap());
    let stub = id.get_stub()?;
    stub.fetch_with_str("https://fake-host/hello").await
}

#[worker::send]
pub async fn handle_storage(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.durable_object("MY_CLASS")?;
    let stub = namespace.id_from_name("singleton")?.get_stub()?;
    stub.fetch_with_str("https://fake-host/storage").await
}
