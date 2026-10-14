use super::SomeSharedData;
use crate::util::{error, ok, url};
use crate::Env;
use serde::{Deserialize, Serialize};
use ts_gen_worker::*;
use wasm_bindgen::prelude::*;

macro_rules! kv_assert_eq {
    ($left: expr, $right: expr) => {{
        let left = &$left;
        let right = &$right;
        if left != right {
            Err(Error::from(format!("{:#?} != {:#?}", left, right)))
        } else {
            Ok(())
        }
    }};
}

pub async fn handle_post_key_value(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let uri = url(&req)?;
    let mut segments = uri.path_segments().unwrap();
    let key = segments.nth(1);
    let value = segments.next();
    let kv = env.some_namespace();
    if let Some(key) = key {
        if let Some(value) = value {
            kv.put(key, value).await?;
        }
    }

    Ok(Response::json(&kv.list().await?.into())?)
}

async fn test_store(env: &Env) -> KVNamespace {
    env.test()
}

pub async fn get(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let store = test_store(&env).await;
    let value = store.get("simple").await?.into_option();
    match value {
        Some(e) => ok(String::from(e)),
        None => error("not found", 404),
    }
}

pub async fn get_not_found(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let store = test_store(&env).await;
    let value = store.get("not_found").await?.into_option();
    match value {
        Some(_) => error("unexpected value present", 500),
        None => ok("passed"),
    }
}

pub async fn list_keys(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let store = test_store(&env).await;
    let list_res = store.list().await?;

    kv_assert_eq!(list_res.keys().len(), 1)?;

    ok("passed")
}

pub async fn put_simple(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let store = test_store(&env).await;
    store.put("put_a", "test").await?;

    let val = String::from(store.get("put_a").await?.unwrap());
    kv_assert_eq!(val, "test")?;

    ok("passed")
}

pub async fn put_metadata(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let store = test_store(&env).await;
    store
        .put_with_str_and_options(
            "put_b",
            "test",
            &KVNamespacePutOptions::builder()
                .metadata(&100.0.into())
                .build(),
        )
        .await?;

    let result = store.get_with_metadata("put_b").await?;
    kv_assert_eq!(result.value().unwrap(), "test")?;
    kv_assert_eq!(result.metadata().as_f64().unwrap() as usize, 100)?;

    ok("passed")
}

pub async fn put_expiration(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    const EXPIRATION: u64 = 2_000_000_000;
    let store = test_store(&env).await;
    store
        .put_with_str_and_options(
            "put_c",
            "test",
            &KVNamespacePutOptions::builder()
                .expiration(EXPIRATION as f64)
                .build(),
        )
        .await?;

    let val = String::from(store.get("put_c").await?.unwrap());
    kv_assert_eq!(val, "test")?;

    let list = store
        .list_with_options(&KVNamespaceListOptions::builder().prefix("put_c").build())
        .await?;
    let key = list
        .keys()
        .into_iter()
        .find(|key| key.name() == "put_c")
        .unwrap();
    kv_assert_eq!(key.expiration().map(|e| e as u64), Some(EXPIRATION))?;

    ok("passed")
}

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

    let store = test_store(&env).await;
    store
        .put_with_str_and_options(
            "put_d",
            "test",
            &KVNamespacePutOptions::builder()
                .metadata(&serde_wasm_bindgen::to_value(&put_meta).map_err(|e| e.to_string())?)
                .build(),
        )
        .await?;

    let result = store.get_with_metadata("put_d").await?;
    let meta: TestStruct =
        serde_wasm_bindgen::from_value(result.metadata()).map_err(|e| e.to_string())?;

    kv_assert_eq!(result.value().unwrap(), "test")?;
    kv_assert_eq!(meta, put_meta)?;

    ok("passed")
}

async fn get_bulk_text(store: &KVNamespace, keys: &[&str]) -> Result<js_sys::Map> {
    let keys: Vec<String> = keys.iter().map(|k| (*k).to_owned()).collect();
    Ok(store.get_with_slice(&keys).await?.unchecked_into())
}

pub async fn get_bulk(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let store = test_store(&env).await;
    store.put("bulk_a", "value_a").await?;
    store.put("bulk_b", "value_b").await?;

    let values = get_bulk_text(&store, &["bulk_a", "bulk_b", "bulk_missing"]).await?;
    kv_assert_eq!(
        values.get(&"bulk_a".into()).as_string(),
        Some("value_a".to_string())
    )?;
    kv_assert_eq!(
        values.get(&"bulk_b".into()).as_string(),
        Some("value_b".to_string())
    )?;
    kv_assert_eq!(values.get(&"bulk_missing".into()).as_string(), None)?;

    ok("passed")
}

pub async fn get_bulk_empty(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let store = test_store(&env).await;
    let values = get_bulk_text(&store, &[]).await?;
    kv_assert_eq!(values.size(), 0)?;

    ok("passed")
}

pub async fn get_bulk_limit(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let store = test_store(&env).await;
    let keys: Vec<String> = (0..101).map(|i| format!("key_{i}")).collect();
    let key_refs: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();

    match get_bulk_text(&store, &key_refs).await {
        Err(_) => ok("passed"),
        Ok(_) => error("expected error for >100 keys", 500),
    }
}
