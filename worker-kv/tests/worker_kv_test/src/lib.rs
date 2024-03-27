use std::future::Future;

use worker::*;
use worker_kv::{KvError, KvStore};

type TestResult = std::result::Result<String, TestError>;

mod utils;

macro_rules! kv_assert_eq {
    ($left: expr, $right: expr) => {{
        let left = &$left;
        let right = &$right;
        if left != right {
            Err(TestError::Other(format!("{:#?} != {:#?}", left, right)))
        } else {
            Ok(())
        }
    }};
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // Create the KV store directly from `worker_kv` as the rust worker sdk uses a published version.
    let store = KvStore::from_this(&env, "test").expect("test kv store not bound");

    Router::with_data(store)
        .get_async("/get", |req, ctx| wrap(req, ctx, get))
        .get_async("/get-not-found", |req, ctx| wrap(req, ctx, get_not_found))
        .get_async("/list-keys", |req, ctx| wrap(req, ctx, list_keys))
        .get_async("/put-simple", |req, ctx| wrap(req, ctx, put_simple))
        .get_async("/put-metadata", |req, ctx| wrap(req, ctx, put_metadata))
        .get_async("/put-expiration", |req, ctx| wrap(req, ctx, put_expiration))
        .run(req, env)
        .await
}

async fn get(_: Request, ctx: RouteContext<KvStore>) -> TestResult {
    let store = ctx.data;
    store
        .get("simple")
        .text()
        .await
        .map_err(TestError::from)
        .and_then(|v| match v {
            Some(e) => Ok(e),
            None => Err(TestError::Other("no value found".into())),
        })
}

async fn get_not_found(_: Request, ctx: RouteContext<KvStore>) -> TestResult {
    let store = ctx.data;
    let value = store.get("not_found").text().await;

    value.map_err(TestError::from).and_then(|v| match v {
        Some(_) => Err(TestError::Other("unexpected value present".into())),
        None => Ok("passed".into()),
    })
}

async fn list_keys(_: Request, ctx: RouteContext<KvStore>) -> TestResult {
    let store = ctx.data;
    let list_res = store.list().execute().await?;

    // TODO: Test cursor and things.
    kv_assert_eq!(list_res.keys.len(), 1)?;

    Ok("passed".into())
}

async fn put_simple(_: Request, ctx: RouteContext<KvStore>) -> TestResult {
    let store = ctx.data;
    store.put("put_a", "test")?.execute().await?;

    let val = store.get("put_a").text().await?.unwrap();
    kv_assert_eq!(val, "test")?;

    Ok("passed".into())
}

async fn put_metadata(_: Request, ctx: RouteContext<KvStore>) -> TestResult {
    let store = ctx.data;
    store.put("put_b", "test")?.metadata(100)?.execute().await?;

    let (val, meta) = store.get("put_b").text_with_metadata::<usize>().await?;
    kv_assert_eq!(val.unwrap(), "test")?;
    kv_assert_eq!(meta.unwrap(), 100)?;

    Ok("passed".into())
}

async fn put_expiration(_: Request, ctx: RouteContext<KvStore>) -> TestResult {
    const EXPIRATION: u64 = 2000000000;
    let store = ctx.data;
    store
        .put("put_c", "test")?
        .expiration(EXPIRATION)
        .execute()
        .await?;

    let val = store.get("put_a").text().await?.unwrap();
    kv_assert_eq!(val, "test")?;

    let list = store.list().prefix("put_c".into()).execute().await?;
    let key = list
        .keys
        .into_iter()
        .find(|key| key.name == "put_c")
        .unwrap();
    kv_assert_eq!(key.expiration, Some(EXPIRATION))?;

    Ok("passed".into())
}

async fn wrap<T>(
    req: Request,
    ctx: RouteContext<KvStore>,
    func: fn(Request, RouteContext<KvStore>) -> T,
) -> Result<Response>
where
    T: Future<Output = TestResult> + 'static,
{
    let result = func(req, ctx);

    match result.await {
        Ok(value) => Response::ok(value),
        Err(e) => Response::ok(e.to_string()).map(|res| res.with_status(500)),
    }
}

#[derive(Debug, thiserror::Error)]
enum TestError {
    #[error("{0}")]
    Kv(#[from] KvError),
    #[error("{0}")]
    Other(String),
}
