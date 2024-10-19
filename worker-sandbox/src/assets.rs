#[cfg(not(feature = "http"))]
pub async fn handle_asset(
    req: worker::Request,
    env: worker::Env,
    _data: crate::SomeSharedData,
) -> worker::Result<worker::Response> {
    use worker::Url;

    let url: Url = req.url()?;
    let name: String = url.path_segments().unwrap().nth(1).unwrap().to_string();
    let url: String = ["https://dummyurl.com/", &name].concat();
    Ok(env
        .assets("ASSETS")
        .expect("ASSETS BINDING")
        .fetch(url, None)
        .await?)
}
