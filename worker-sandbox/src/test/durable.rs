use crate::ensure;
use worker::*;

#[allow(dead_code)]
pub async fn basic_test(env: &Env) -> Result<()> {
    let namespace: ObjectNamespace = env.durable_object("MY_CLASS")?;
    let id = namespace.id_from_name("A")?;
    ensure!(id.name() == Some("A".into()), "Missing name");
    ensure!(
        namespace.unique_id()?.name().is_none(),
        "Expected name property to be absent"
    );
    let bad = env.durable_object("DFSDF_FAKE_BINDING");
    ensure!(bad.is_err(), "Invalid binding did not raise error");

    let stub = id.get_stub()?;
    let res = stub.fetch_with_str("hello").await?.text().await?;
    let res2 = stub
        .fetch_with_request(Request::new_with_init(
            "hello",
            RequestInit::new()
                .with_body(Some("lol".into()))
                .with_method(Method::Post),
        )?)
        .await?
        .text()
        .await?;

    ensure!(res == res2, "Durable object responded wrong to 'hello'");

    let res = stub.fetch_with_str("storage").await?.text().await?;
    let num = res
        .parse::<usize>()
        .map_err(|_| "Durable Object responded wrong to 'storage': ".to_string() + &res)?;
    let res = stub.fetch_with_str("storage").await?.text().await?;
    let num2 = res
        .parse::<usize>()
        .map_err(|_| "Durable Object responded wrong to 'storage'".to_string())?;

    ensure!(
        num2 == num + 1,
        "Durable object responded wrong to 'storage'"
    );

    let res = stub.fetch_with_str("transaction").await?.text().await?;
    let num = res
        .parse::<usize>()
        .map_err(|_| "Durable Object responded wrong to 'transaction': ".to_string() + &res)?;

    ensure!(
        num == num2 + 1,
        "Durable object responded wrong to 'storage'"
    );

    Ok(())
}
