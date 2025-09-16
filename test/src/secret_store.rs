use crate::SomeSharedData;
use worker::{Env, Request, Response, Result};

#[worker::send]
pub async fn get_from_secret_store(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let secrets = env.secret_store("SECRETS")?;
    let secret_value = secrets.get().await?;

    match secret_value {
        Some(_value) => Response::ok("secret value"),
        None => Response::error("Secret not found", 404),
    }
}

#[worker::send]
pub async fn get_from_secret_store_missing(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let secrets = env.secret_store("MISSING_SECRET")?;
    let secret_value = secrets.get().await?;

    match secret_value {
        Some(value) => Response::ok(value),
        None => Response::error("Secret not found", 500),
    }
}
