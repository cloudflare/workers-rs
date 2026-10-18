use crate::{DataKind, DurableObjectState, Error, Request, Response, Result, WebSocket};

/// A Durable Object class. Implement on a struct marked `#[durable_object]`, which
/// exports the struct as the JS class and forwards the platform's calls here.
///
/// `Env` is the app's generated environment type. The `websocket_*` handlers are the
/// hibernation API callbacks; `message` is the generated `string | ArrayBuffer` union,
/// the same type `MessageEvent::data()` returns.
#[allow(async_fn_in_trait)]
pub trait DurableObject: has_durable_object_attribute + Sized {
    type Env;

    fn new(state: DurableObjectState, env: Self::Env) -> Self;

    async fn fetch(&self, req: Request) -> Result<Response>;

    async fn alarm(&self) -> Result<()> {
        unimplemented!("alarm() handler not implemented")
    }

    /// A hibernated socket's message is `String` or `ArrayBuffer`, never the other arms.
    #[allow(unused_variables)]
    async fn websocket_message(&self, ws: WebSocket, message: DataKind) -> Result<()> {
        unimplemented!("websocket_message() handler not implemented")
    }

    #[allow(unused_variables)]
    async fn websocket_close(
        &self,
        ws: WebSocket,
        code: usize,
        reason: String,
        was_clean: bool,
    ) -> Result<()> {
        unimplemented!("websocket_close() handler not implemented")
    }

    #[allow(unused_variables)]
    async fn websocket_error(&self, ws: WebSocket, error: Error) -> Result<()> {
        unimplemented!("websocket_error() handler not implemented")
    }
}

/// Only `#[durable_object]` implements this, so a `DurableObject` impl whose struct
/// lacks the attribute (and so exports no class) fails to compile with the attribute's
/// name in the error.
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub trait has_durable_object_attribute {}
