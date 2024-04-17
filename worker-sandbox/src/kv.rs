use super::SomeSharedData;
use serde::{Deserialize, Serialize};
use worker::{console_debug, Env, Request, Response, Result};

macro_rules! kv_assert_eq {
    ($left: expr, $right: expr) => {{
        let left = &$left;
        let right = &$right;
        if left != right {
            Err(worker::Error::RustError(format!(
                "{:#?} != {:#?}",
                left, right
            )))
        } else {
            Ok(())
        }
    }};
}

#[worker::send]
pub async fn handle_post_key_value(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let uri = req.url()?;
    let mut segments = uri.path_segments().unwrap();
    let key = segments.nth(1);
    let value = segments.next();
    let kv = env.kv("SOME_NAMESPACE")?;
    if let Some(key) = key {
        if let Some(value) = value {
            kv.put(key, value)?.execute().await?;
        }
    }

    Response::from_json(&kv.list().execute().await?)
}

const TEST_NAMESPACE: &str = "TEST";

#[worker::send]
pub async fn get(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let store = env.kv(TEST_NAMESPACE)?;
    let value = store.get("simple").text().await?;
    match value {
        Some(e) => Response::ok(e),
        None => Response::error("not found", 404),
    }
}

#[worker::send]
pub async fn get_not_found(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let store = env.kv(TEST_NAMESPACE)?;
    let value = store.get("not_found").text().await?;
    match value {
        Some(_) => Response::error("unexpected value present", 500),
        None => Response::ok("passed"),
    }
}

#[worker::send]
pub async fn list_keys(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let store = env.kv(TEST_NAMESPACE)?;
    let list_res = store.list().execute().await?;

    // TODO: Test cursor and things.
    kv_assert_eq!(list_res.keys.len(), 1)?;

    Response::ok("passed")
}

#[worker::send]
pub async fn put_simple(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let store = env.kv(TEST_NAMESPACE)?;
    store.put("put_a", "test")?.execute().await?;

    let val = store.get("put_a").text().await?.unwrap();
    kv_assert_eq!(val, "test")?;

    Response::ok("passed")
}

#[worker::send]
pub async fn put_metadata(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let store = env.kv(TEST_NAMESPACE)?;
    store.put("put_b", "test")?.metadata(100)?.execute().await?;

    let (val, meta) = store.get("put_b").text_with_metadata::<usize>().await?;
    kv_assert_eq!(val.unwrap(), "test")?;
    kv_assert_eq!(meta.unwrap(), 100)?;

    Response::ok("passed")
}

#[worker::send]
pub async fn put_metadata_struct(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    #[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
    pub struct TestStruct {
        pub a: String,
        pub b: usize,
    }

    let store = env.kv(TEST_NAMESPACE)?;
    store
        .put("put_b", "test")?
        .metadata(TestStruct::default())?
        .execute()
        .await?;

    let (val, meta) = store
        .get("put_b")
        .text_with_metadata::<serde_json::Value>()
        .await?;

    kv_assert_eq!(val.unwrap(), "test")?;
    console_debug!("{:?}", meta);
    assert!(meta.is_some());
    // kv_assert_eq!(meta.unwrap(), TestStruct::default())?;

    Response::ok("passed")
}

#[worker::send]
pub async fn put_expiration(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    const EXPIRATION: u64 = 2000000000;
    let store = env.kv(TEST_NAMESPACE)?;
    store
        .put("put_c", "test")?
        .expiration(EXPIRATION)
        .execute()
        .await?;

    let val = store.get("put_c").text().await?.unwrap();
    kv_assert_eq!(val, "test")?;

    let list = store.list().prefix("put_c".into()).execute().await?;
    let key = list
        .keys
        .into_iter()
        .find(|key| key.name == "put_c")
        .unwrap();
    kv_assert_eq!(key.expiration, Some(EXPIRATION))?;

    Response::ok("passed")
}
