use super::SomeSharedData;
use serde::{Deserialize, Serialize};
use worker::{Env, Request, Response, Result};

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
pub async fn put_expiration(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    const EXPIRATION: u64 = 2_000_000_000;
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

#[worker::send]
pub async fn put_metadata_struct(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    #[derive(Debug, Default, Deserialize, Serialize, PartialEq, Clone)]
    pub struct TestStruct {
        pub a: String,
        pub b: usize,
    }

    let put_meta = TestStruct::default();

    let store = env.kv(TEST_NAMESPACE)?;
    store
        .put("put_d", "test")?
        .metadata(put_meta.clone())?
        .execute()
        .await?;

    let (val, meta) = store
        .get("put_d")
        .text_with_metadata::<TestStruct>()
        .await?;

    kv_assert_eq!(val.unwrap(), "test")?;
    kv_assert_eq!(meta.unwrap(), put_meta)?;

    Response::ok("passed")
}

#[worker::send]
pub async fn get_bulk(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let store = env.kv(TEST_NAMESPACE)?;
    store.put("bulk_a", "value_a")?.execute().await?;
    store.put("bulk_b", "value_b")?.execute().await?;

    let values = store
        .get_bulk(&["bulk_a", "bulk_b", "bulk_missing"])
        .text()
        .await?;
    kv_assert_eq!(values.get("bulk_a").unwrap(), &Some("value_a".to_string()))?;
    kv_assert_eq!(values.get("bulk_b").unwrap(), &Some("value_b".to_string()))?;
    kv_assert_eq!(values.get("bulk_missing").unwrap(), &None)?;

    Response::ok("passed")
}

#[worker::send]
pub async fn get_bulk_empty(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let store = env.kv(TEST_NAMESPACE)?;
    let empty: &[&str] = &[];
    let values = store.get_bulk(empty).text().await?;
    kv_assert_eq!(values.len(), 0)?;

    Response::ok("passed")
}

#[worker::send]
pub async fn get_bulk_limit(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let store = env.kv(TEST_NAMESPACE)?;
    let keys: Vec<String> = (0..101).map(|i| format!("key_{i}")).collect();
    let key_refs: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();

    match store.get_bulk(&key_refs).text().await {
        Err(_) => Response::ok("passed"),
        Ok(_) => Response::error("expected error for >100 keys", 500),
    }
}
