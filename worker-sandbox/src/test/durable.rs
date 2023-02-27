use crate::ensure;
use worker::*;

#[allow(dead_code)]
pub async fn basic_test(env: &Env) -> Result<()> {
    let namespace: ObjectNamespace = env.durable_object("MY_CLASS")?;
    let id = namespace.id_from_name("A")?;
    let bad = env.durable_object("DFSDF_FAKE_BINDING");
    ensure!(bad.is_err(), "Invalid binding did not raise error");

    let stub = id.get_stub()?;
    let res = stub
        .fetch_with_str("hello")
        .await?
        .into_body()
        .bytes()
        .await
        .map_err(|_| Error::BadEncoding)?;

    let res2 = stub
        .fetch_with_request(
            http::Request::builder()
                .method(http::Method::POST)
                .uri("hello")
                .body("lol".into())
                .unwrap(),
        )
        .await?
        .into_body()
        .bytes()
        .await
        .map_err(|_| Error::BadEncoding)?;

    ensure!(res == res2, "Durable object responded wrong to 'hello'");

    let res = stub
        .fetch_with_str("storage")
        .await?
        .into_body()
        .bytes()
        .await
        .map_err(|_| Error::BadEncoding)?;

    let num = std::str::from_utf8(&res)
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .ok_or_else(|| "Durable Object responded wrong to 'storage'".to_string())?;

    let res = stub
        .fetch_with_str("storage")
        .await?
        .into_body()
        .bytes()
        .await
        .map_err(|_| Error::BadEncoding)?;

    let num2 = std::str::from_utf8(&res)
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .ok_or_else(|| "Durable Object responded wrong to 'storage'".to_string())?;

    ensure!(
        num2 == num + 1,
        "Durable object responded wrong to 'storage'"
    );

    let res = stub
        .fetch_with_str("transaction")
        .await?
        .into_body()
        .bytes()
        .await
        .map_err(|_| Error::BadEncoding)?;

    let num = std::str::from_utf8(&res)
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .ok_or_else(|| {
            "Durable Object responded wrong to 'transaction': ".to_string()
                + std::str::from_utf8(&res).unwrap_or("<malformed>")
        })?;

    ensure!(
        num == num2 + 1,
        "Durable object responded wrong to 'storage'"
    );

    Ok(())
}
