use futures_util::TryStreamExt;
use wasm_bindgen::JsCast;

pub use crate::bindings::email::*;
use crate::{send::SendFuture, ByteStream, EnvBinding, Headers, Result};

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

/// High-level wrapper around the inbound `ForwardableEmailMessage` handed to
/// an `#[event(email)]` handler. Returns `worker::Headers` / `ByteStream` and
/// `Send`-safe futures so the handler composes with async frameworks like
/// axum that require `Send` bounds.
#[derive(Debug)]
pub struct InboundEmail {
    pub inner: ForwardableEmailMessage,
}

impl InboundEmail {
    /// Envelope `From` of the inbound message.
    pub fn from_email(&self) -> String {
        self.inner.from()
    }

    /// Envelope `To` of the inbound message.
    pub fn to_email(&self) -> String {
        self.inner.to()
    }

    /// Headers of the inbound message.
    pub fn headers(&self) -> Headers {
        Headers(self.inner.headers())
    }

    /// Stream of the raw email content.
    pub fn raw(&self) -> ByteStream {
        ByteStream {
            inner: wasm_streams::ReadableStream::from_raw(self.inner.raw()).into_stream(),
        }
    }

    /// Convenience: collect the raw email content into a `Vec<u8>`.
    pub async fn raw_bytes(&self) -> Result<Vec<u8>> {
        self.raw()
            .try_fold(Vec::new(), |mut bytes, mut chunk| async move {
                bytes.append(&mut chunk);
                Ok(bytes)
            })
            .await
    }

    /// Size of the raw email content in bytes.
    pub fn raw_size(&self) -> f64 {
        self.inner.raw_size()
    }

    /// Reject this message with a permanent SMTP error.
    pub fn reject(&self, reason: &str) {
        self.inner.set_reject(reason)
    }

    /// Forward this message to a verified destination address.
    pub async fn forward(
        &self,
        recipient: &str,
        headers: Option<Headers>,
    ) -> Result<EmailSendResult> {
        let fut = SendFuture::new(async move {
            match headers {
                Some(h) => self.inner.forward_with_headers(recipient, &h.0).await,
                None => self.inner.forward(recipient).await,
            }
        });
        Ok(fut.await?)
    }

    /// Reply to the sender of this message with a new `EmailMessage`.
    pub async fn reply(&self, message: &EmailMessage) -> Result<EmailSendResult> {
        let fut = SendFuture::new(async move { self.inner.reply(message).await });
        Ok(fut.await?)
    }
}

#[cfg(test)]
mod send_check {
    // `SendEmail` and `InboundEmail` are `Send` automatically —
    // wasm-bindgen makes `JsValue` `Send + Sync` and every extern `pub type`
    // carries that through. This compile-time check guards against an
    // upstream regression.
    use super::{InboundEmail, SendEmail};
    fn _assert_send<T: Send>() {}
    #[allow(dead_code)]
    fn _check() {
        _assert_send::<SendEmail>();
        _assert_send::<InboundEmail>();
    }
}
