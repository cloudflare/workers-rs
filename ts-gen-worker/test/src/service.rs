use super::SomeSharedData;

use crate::Env;
use ts_gen_worker::*;

pub async fn handle_remote_by_request(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let fetcher = env.remote();
    fetcher.fetch(&req).await
}

pub async fn handle_remote_by_path(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let fetcher = env.remote();
    let init = RequestInit::builder().method("POST").build();
    fetcher.fetch_with_str_and_init(req.url(), &init).await
}
