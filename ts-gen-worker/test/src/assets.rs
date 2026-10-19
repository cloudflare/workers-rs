use crate::Env;
use ts_gen_worker::*;

pub async fn handle_asset(
    req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let url = req.url_parsed()?;
    let name: String = url.path_segments().unwrap().nth(1).unwrap().to_string();
    let url: String = ["https://dummyurl.com/", &name].concat();

    env.assets().fetch_with_str(&url).await
}
