use crate::SomeSharedData;
use worker::{Env, Request, Response, Result};

pub async fn get_from_secret_store(
    _req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let secrets = env.secret_store("SECRETS")?;
    let secret_value = secrets.get().await?;

    match secret_value {
        Some(value) => Response::ok(value),
        None => Response::error("Secret not found", 404),
    }
}

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

// Compile-time assertion: public async SecretStore methods return Send futures.
#[allow(dead_code, unused)]
fn _assert_send() {
    fn require_send<T: Send>(_t: T) {}
    fn secret_store(ss: worker::SecretStore) {
        require_send(ss.get());
    }
}
