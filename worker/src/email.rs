use futures_util::TryStreamExt;
use wasm_bindgen::JsCast;

pub use crate::bindings::email::*;
use crate::{ByteStream, EnvBinding, Result};

impl EnvBinding for SendEmail {
    const TYPE_NAME: &'static str = "SendEmail";

    // `SendEmail` is a TypeScript interface, not a class — the runtime
    // doesn't expose a `SendEmail` global for the default
    // `constructor.name` check to match against. The TS types are
    // authoritative: if `env.EMAIL` is bound to a SendEmail per
    // `wrangler.toml`, the runtime hands us the right shape, so we
    // skip the check and `unchecked_into`.
    fn get(val: wasm_bindgen::JsValue) -> Result<Self> {
        Ok(val.unchecked_into())
    }
}

impl ForwardableEmailMessage {
    /// Stream of the raw email content.
    pub fn raw_byte_stream(&self) -> ByteStream {
        self.raw().into()
    }

    /// Convenience: collect the raw email content into a `Vec<u8>`.
    pub async fn raw_bytes(&self) -> Result<Vec<u8>> {
        Into::<ByteStream>::into(self.raw())
            .try_fold(Vec::new(), |mut bytes, mut chunk| async move {
                bytes.append(&mut chunk);
                Ok(bytes)
            })
            .await
    }
}

#[cfg(test)]
mod send_check {
    // `SendEmail` and `InboundEmail` are `Send` automatically —
    // wasm-bindgen makes `JsValue` `Send + Sync` and every extern `pub type`
    // carries that through. This compile-time check guards against an
    // upstream regression.
    use super::{ForwardableEmailMessage, SendEmail};
    fn _assert_send<T: Send>() {}
    #[allow(dead_code)]
    fn _check() {
        _assert_send::<SendEmail>();
        _assert_send::<ForwardableEmailMessage>();
    }
}
