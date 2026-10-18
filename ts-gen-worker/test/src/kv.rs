use super::SomeSharedData;
use crate::Env;
use serde::{Deserialize, Serialize};
use ts_gen_worker::*;

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
    let uri = req.url_parsed()?;
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
    match store.get("simple").await? {
        Some(e) => Response::builder().ok(e),
        None => Response::builder().error("not found", 404),
    }
}

pub async fn get_not_found(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let store = test_store(&env).await;
    match store.get("not_found").await? {
        Some(_) => Response::builder().error("unexpected value present", 500),
        None => Response::builder().ok("passed"),
    }
}

pub async fn list_keys(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let store = test_store(&env).await;
    let list_res = store.list().await?;

    kv_assert_eq!(list_res.keys().len(), 1)?;

    Response::builder().ok("passed")
}

pub async fn put_simple(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let store = test_store(&env).await;
    store.put("put_a", "test").await?;

    let val = store.get("put_a").await?.unwrap();
    kv_assert_eq!(val, "test")?;

    Response::builder().ok("passed")
}

pub async fn put_metadata(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let store = test_store(&env).await;
    store
        .put_with_options(
            "put_b",
            "test",
            &KVNamespacePutOptions::builder()
                .metadata(&100.0.into())
                .build(),
        )
        .await?;

    let result = store.get_with_metadata_typed::<f64>("put_b").await?;
    kv_assert_eq!(result.value().unwrap(), "test")?;
    kv_assert_eq!(result.metadata(), Some(100.0))?;

    Response::builder().ok("passed")
}

pub async fn put_expiration(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    const EXPIRATION: u64 = 2_000_000_000;
    let store = test_store(&env).await;
    store
        .put_with_options(
            "put_c",
            "test",
            &KVNamespacePutOptions::builder()
                .expiration(EXPIRATION)
                .build(),
        )
        .await?;

    let val = store.get("put_c").await?.unwrap();
    kv_assert_eq!(val, "test")?;

    let list = store
        .list_with_options(&KVNamespaceListOptions::builder().prefix("put_c").build())
        .await?;
    let key = list
        .keys()
        .into_iter()
        .find(|key| key.name() == "put_c")
        .unwrap();
    kv_assert_eq!(key.expiration(), Some(EXPIRATION))?;

    Response::builder().ok("passed")
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
        .put_with_options(
            "put_d",
            "test",
            &KVNamespacePutOptions::builder()
                .metadata(&serde_wasm_bindgen::to_value(&put_meta)?)
                .build(),
        )
        .await?;

    let result = store.get_with_metadata("put_d").await?;
    let meta: TestStruct = serde_wasm_bindgen::from_value(result.metadata().unwrap_or_default())?;

    kv_assert_eq!(result.value().unwrap(), "test")?;
    kv_assert_eq!(meta, put_meta)?;

    Response::builder().ok("passed")
}

pub async fn get_bulk(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let store = test_store(&env).await;
    store.put("bulk_a", "value_a").await?;
    store.put("bulk_b", "value_b").await?;

    let values = store
        .get_with_slice(&["bulk_a", "bulk_b", "bulk_missing"])
        .await?;
    let text = |key: &str| values.get(key).and_then(|v| v.into_option());
    kv_assert_eq!(text("bulk_a"), Some("value_a".to_string()))?;
    kv_assert_eq!(text("bulk_b"), Some("value_b".to_string()))?;
    kv_assert_eq!(values.has("bulk_missing"), true)?;
    kv_assert_eq!(text("bulk_missing"), None)?;

    Response::builder().ok("passed")
}

pub async fn get_bulk_empty(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let store = test_store(&env).await;
    let values = store.get_with_slice(&[] as &[&str]).await?;
    kv_assert_eq!(values.size(), 0)?;

    Response::builder().ok("passed")
}

pub async fn get_bulk_limit(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let store = test_store(&env).await;
    let keys: Vec<String> = (0..101).map(|i| format!("key_{i}")).collect();

    match store.get_with_slice(&keys).await {
        Err(_) => Response::builder().ok("passed"),
        Ok(_) => Response::builder().error("expected error for >100 keys", 500),
    }
}
