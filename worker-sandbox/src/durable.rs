use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::{cell::RefCell, collections::HashMap};

use worker::{
    durable_object,
    js_sys::{self, Uint8Array},
    wasm_bindgen::JsValue,
    Env, Method, ObjectNamespace, Request, RequestInit, Response, Result, State,
};

#[durable_object]
pub struct MyClass {
    state: State,
    number: RefCell<usize>,
}

#[derive(Deserialize)]
pub struct QueryParams {
    name: String,
}

impl DurableObject for MyClass {
    fn new(state: State, _env: Env) -> Self {
        // Unfortunately we can't access the `name` property within the Durable Object (see <https://github.com/cloudflare/workerd/issues/2240>). Instead, we can pass it as a request parameter.
        assert!(state.id().name().is_none());
        Self {
            state,
            number: RefCell::new(0),
        }
    }

    #[allow(clippy::too_many_lines)]
    async fn fetch(&self, req: Request) -> Result<Response> {
        let handler = async move {
            match req.path().as_str() {
                "/hello" => {
                    let name = &req.query::<QueryParams>()?.name;
                    Response::ok(format!("Hello from {name}!"))
                }
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
                        .get_multiple(keys)
                        .await
                        .map_err(|e| e.to_string() + " -- get_multiple")?;
                    assert_eq!(
                        serde_wasm_bindgen::from_value::<Option<i32>>(
                            vals.get(&"anything".into())
                        )?,
                        Some(45),
                        "Didn't get the right Option<i32> using get_multiple"
                    );
                    assert_eq!(
                        serde_wasm_bindgen::from_value::<[(String, i32); 2]>(
                            vals.get(&"array".into())
                        )?,
                        [("one".to_string(), 1), ("two".to_string(), 2)],
                        "Didn't get the right array using get_multiple"
                    );
                    assert_eq!(
                        serde_wasm_bindgen::from_value::<HashMap<String, i32>>(
                            vals.get(&"map".into())
                        )?,
                        map,
                        "Didn't get the right HashMap<String, i32> using get_multiple"
                    );

                    {
                        let bytes = Uint8Array::new_with_length(3);
                        bytes.copy_from(b"123");
                        storage.put_raw("bytes", bytes).await?;
                        let bytes = storage.get::<Vec<u8>>("bytes").await?;
                        storage.delete("bytes").await?;
                        assert_eq!(
                            bytes, b"123",
                            "eficient serialization of bytes is not preserved"
                        );
                    }

                    #[allow(clippy::items_after_statements)]
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

                    assert_eq!(
                        storage.get::<String>("thing").await?,
                        "Hello there",
                        "Didn't put the right thing with put_multiple"
                    );
                    assert_eq!(
                        storage.get::<i32>("other").await?,
                        56,
                        "Didn't put the right thing with put_multiple"
                    );

                    storage.delete_multiple(vec!["thing", "other"]).await?;

                    {
                        const BAR: &[u8] = b"bar";
                        let obj = js_sys::Object::new();
                        let value = Uint8Array::new_with_length(u32::try_from(BAR.len()).unwrap());
                        value.copy_from(BAR);
                        js_sys::Reflect::set(&obj, &JsValue::from_str("foo"), &value.into())?;
                        storage.put_multiple_raw(obj).await?;
                        assert_eq!(
                            storage.get::<Vec<u8>>("foo").await?,
                            BAR,
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
    let namespace = env.durable_object("MY_CLASS")?;
    let name = "my-durable-object";
    let id = namespace.id_from_name(name)?;
    let stub = id.get_stub()?;
    stub.fetch_with_str(&format!("https://fake-host/hello?name={name}"))
        .await
}

#[worker::send]
pub async fn handle_hello_unique(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.durable_object("MY_CLASS")?;
    let id = namespace.unique_id()?;
    let name = id.to_string();
    let stub = id.get_stub()?;
    stub.fetch_with_str(&format!("https://fake-host/hello?name={name}"))
        .await
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

#[worker::send]
pub async fn handle_basic_test(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace: ObjectNamespace = env.durable_object("MY_CLASS")?;
    let id = namespace.id_from_name("A")?;
    assert_eq!(id.name(), Some("A".into()), "Missing name");
    assert!(
        namespace.unique_id()?.name().is_none(),
        "Expected name property to be absent"
    );
    let bad = env.durable_object("DFSDF_FAKE_BINDING");
    assert!(bad.is_err(), "Invalid binding did not raise error");

    let stub = id.get_stub()?;
    let res = stub
        .fetch_with_str(&format!(
            "https://fake-host/hello?name={}",
            id.name().unwrap()
        ))
        .await?
        .text()
        .await?;
    let res2 = stub
        .fetch_with_request(Request::new_with_init(
            &format!("https://fake-host/hello?name={}", id.name().unwrap()),
            RequestInit::new()
                .with_body(Some("lol".into()))
                .with_method(Method::Post),
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
    let num = res
        .parse::<usize>()
        .map_err(|_| "Durable Object responded wrong to 'storage': ".to_string() + &res)?;
    let res = stub
        .fetch_with_str("https://fake-host/storage")
        .await?
        .text()
        .await?;
    let num2 = res
        .parse::<usize>()
        .map_err(|_| "Durable Object responded wrong to 'storage'".to_string())?;

    assert_eq!(num2, num + 1, "Durable object responded wrong to 'storage'");

    let res = stub
        .fetch_with_str("https://fake-host/transaction")
        .await?
        .text()
        .await?;
    let num = res
        .parse::<usize>()
        .map_err(|_| "Durable Object responded wrong to 'transaction': ".to_string() + &res)?;

    assert_eq!(
        num,
        num2 + 1,
        "Durable object responded wrong to 'transaction'"
    );

    Response::ok("ok")
}

#[worker::send]
pub async fn handle_get_by_name(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.durable_object("MY_CLASS")?;
    let name = "my-durable-object";
    
    // Using the new get_by_name method - this is equivalent to:
    // let id = namespace.id_from_name(name)?;
    // let stub = id.get_stub()?;
    let stub = namespace.get_by_name(name)?;
    
    stub.fetch_with_str(&format!("https://fake-host/hello?name={name}"))
        .await
}

#[worker::send]
pub async fn handle_get_by_name_with_location_hint(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.durable_object("MY_CLASS")?;
    let name = "my-durable-object";
    
    // Using the new get_by_name_with_location_hint method
    let stub = namespace.get_by_name_with_location_hint(name, "enam")?;
    
    stub.fetch_with_str(&format!("https://fake-host/hello?name={name}"))
        .await
}
