use crate::ensure;
use worker::{durable::ObjectNamespace, *};

pub async fn basic_test(env: &Env) -> Result<()> {
    let namespace: ObjectNamespace = env.get_binding("MY_CLASS")?;
    let id = namespace.id_from_name("A")?;
    let bad = env.get_binding::<ObjectNamespace>("DFSDF");
    ensure!(bad.is_err(), "Invalid binding did not raise error");

    let stub = id.get_stub()?;
    let res = stub.fetch_with_str("hello").await?.text().await?;
    let res2 = stub
        .fetch_with_request(Request::new_with_init(
            "hello",
            RequestInit::new().body(Some(&"lol".into())).method("POST"),
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
