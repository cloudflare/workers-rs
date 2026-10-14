use crate::{DurableObjectState, Request, Response, Result};

/// A Durable Object class. Implement on a struct marked `#[durable_object]`, which
/// exports the struct as the JS class and forwards the platform's calls here.
///
/// `Env` is the app's generated environment type.
#[allow(async_fn_in_trait)]
pub trait DurableObject: Sized {
    type Env;

    fn new(state: DurableObjectState, env: Self::Env) -> Self;

    async fn fetch(&self, req: Request) -> Result<Response>;

    async fn alarm(&self) -> Result<()> {
        unimplemented!("alarm() handler not implemented")
    }
}
