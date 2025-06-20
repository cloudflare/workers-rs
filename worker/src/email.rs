use futures_util::TryStreamExt;
use wasm_bindgen_futures::JsFuture;
use web_sys::ReadableStream;
use worker_sys::EmailMessage as EmailMessageSys;

use crate::{send::SendFuture, ByteStream, Headers, Result};

pub struct EmailMessage {
    pub inner: EmailMessageSys,
}

impl EmailMessage {
    /// construct a new email message
    pub fn new(from: &str, to: &str, raw: &str) -> Result<Self> {
        Ok(EmailMessage {
            inner: EmailMessageSys::new(from, to, raw)?,
        })
    }

    /// construct a new email message for a ReadableStream
    pub fn new_from_stream(from: &str, to: &str, raw: &ReadableStream) -> Result<Self> {
        Ok(EmailMessage {
            inner: EmailMessageSys::new_from_stream(from, to, raw)?,
        })
    }

    /// the from field of the email message
    pub fn from_email(&self) -> String {
        self.inner.from().unwrap().into()
    }

    /// the to field of the email message
    pub fn to_email(&self) -> String {
        self.inner.to().unwrap().into()
    }

    /// the headers field of the email message
    pub fn headers(&self) -> Headers {
        Headers(self.inner.headers().unwrap())
    }

    /// the raw email message
    pub fn raw(&self) -> Result<ByteStream> {
        self.inner.raw().map_err(Into::into).map(|rs| ByteStream {
            inner: wasm_streams::ReadableStream::from_raw(rs).into_stream(),
        })
    }

    pub async fn raw_bytes(&self) -> Result<Vec<u8>> {
        self.raw()?
            .try_fold(Vec::new(), |mut bytes, mut chunk| async move {
                bytes.append(&mut chunk);
                Ok(bytes)
            })
            .await
    }

    /// the raw size of the message
    pub fn raw_size(&self) -> f64 {
        self.inner.raw_size().unwrap().into()
    }

    /// reject message with reason
    pub fn reject(&self, reason: String) {
        self.inner.set_reject(reason.into()).unwrap()
    }

    /// forward message to recipient
    pub async fn forward(&self, recipient: String, headers: Option<Headers>) -> Result<()> {
        let promise = self.inner.forward(recipient.into(), headers.map(|h| h.0))?;

        let fut = SendFuture::new(JsFuture::from(promise));
        fut.await?;
        Ok(())
    }

    /// reply with email message to recipient
    pub async fn reply(&self, message: EmailMessage) -> Result<()> {
        let promise = self.inner.reply(message.inner)?;

        let fut = SendFuture::new(JsFuture::from(promise));
        fut.await?;
        Ok(())
    }
}
