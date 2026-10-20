use crate::Env;
use crate::SomeSharedData;
use ts_gen_worker::*;

pub async fn get_from_secret_store(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let secrets = env.secrets();
    let secret_value = secrets.get().await?;
    Response::builder().ok(secret_value)
}

pub async fn get_from_secret_store_missing(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let secrets = env.missing_secret();
    let secret_value = secrets.get().await?;
    Response::builder().ok(secret_value)
}
