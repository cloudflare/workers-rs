use serde::Serialize;
use std::collections::HashMap;

use worker::*;

use crate::ensure;

#[durable_object]
pub struct MyClass {
    state: worker::durable::State,
    number: usize,
}

#[durable_object]
impl DurableObject for MyClass {
    fn constructor(state: worker::durable::State, _env: Env) -> Self {
        Self { state, number: 0 }
    }

    async fn fetch(&mut self, req: Request) -> Result<Response> {
        let handler = async move {
            match req.path().as_str() {
                "/hello" => Response::ok("Hello!"),
                "/storage" => {
                    let mut storage = self.state.storage();
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
                        format!("Didn't list all of the keys: {:?}", keys)
                    );
                    let vals = storage
                        .get_multiple(keys)
                        .await
                        .map_err(|e| e.to_string() + " -- get_multiple")?;
                    ensure!(
                        vals.get(&"anything".into()).into_serde::<Option<i32>>()? == Some(45),
                        "Didn't get the right Option<i32> using get_multiple"
                    );
                    ensure!(
                        vals.get(&"array".into())
                            .into_serde::<[(String, i32); 2]>()?
                            == [("one".to_string(), 1), ("two".to_string(), 2)],
                        "Didn't get the right array using get_multiple"
                    );
                    ensure!(
                        vals.get(&"map".into())
                            .into_serde::<HashMap<String, i32>>()?
                            == map,
                        "Didn't get the right HashMap<String, i32> using get_multiple"
                    );

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

                    self.number = storage.get("count").await.unwrap_or(0) + 1;

                    storage.delete_all().await?;

                    storage.put("count", self.number).await?;
                    Response::ok(self.number.to_string())
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
