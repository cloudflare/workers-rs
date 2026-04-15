use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use worker_sys::{EmailMessage as EmailMessageSys, SendEmail as SendEmailSys};

use crate::{send::SendFuture, EnvBinding, Result};

/// A binding to the [Cloudflare Email Sending] service, declared under
/// `[[send_email]]` in `wrangler.toml` and retrieved via
/// [`Env::send_email`](crate::Env::send_email).
///
/// Build an [`EmailMessage`] (optionally with a MIME builder such as
/// [`mail-builder`]) and hand it to [`SendEmail::send`].
///
/// [Cloudflare Email Sending]: https://developers.cloudflare.com/email-routing/email-workers/send-email-workers/
/// [`mail-builder`]: https://crates.io/crates/mail-builder
///
/// ```ignore
/// use worker::*;
///
/// #[event(fetch)]
/// async fn fetch(_req: Request, env: Env, _ctx: Context) -> Result<Response> {
///     let raw = "From: sender@example.com\r\n\
///                To: recipient@example.com\r\n\
///                Subject: Hello\r\n\
///                \r\n\
///                Hi there!";
///     let message = EmailMessage::new(
///         "sender@example.com",
///         "recipient@example.com",
///         raw,
///     )?;
///     env.send_email("SEND_EMAIL")?.send(&message).await?;
///     Response::ok("sent")
/// }
/// ```
#[derive(Debug)]
pub struct SendEmail(SendEmailSys);

unsafe impl Send for SendEmail {}
unsafe impl Sync for SendEmail {}

impl EnvBinding for SendEmail {
    const TYPE_NAME: &'static str = "SendEmail";

    // `SendEmail` is a TypeScript interface (not a class) in
    // @cloudflare/workers-types, so the runtime doesn't expose a `SendEmail`
    // global for the default `constructor.name` check to match against. Skip
    // the check and accept whatever object the runtime hands us.
    fn get(val: JsValue) -> Result<Self> {
        Ok(val.unchecked_into())
    }
}

impl JsCast for SendEmail {
    // `SendEmail` has no JS class at runtime (see `EnvBinding::get` above), so
    // the wasm-bindgen-generated `val instanceof SendEmail` shim would throw a
    // `ReferenceError` if we ever reached it. Fall back to a plain object
    // check — good enough for the binding and can't blow up.
    fn instanceof(val: &JsValue) -> bool {
        val.is_object()
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        Self(val.into())
    }

    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}

impl AsRef<JsValue> for SendEmail {
    fn as_ref(&self) -> &JsValue {
        &self.0
    }
}

impl From<SendEmail> for JsValue {
    fn from(sender: SendEmail) -> Self {
        JsValue::from(sender.0)
    }
}

impl From<SendEmailSys> for SendEmail {
    fn from(inner: SendEmailSys) -> Self {
        Self(inner)
    }
}

impl SendEmail {
    /// Dispatch a prebuilt [`EmailMessage`] through this binding.
    pub async fn send(&self, message: &EmailMessage) -> Result<()> {
        let promise = self.0.send(&message.0)?;
        SendFuture::new(JsFuture::from(promise)).await?;
        Ok(())
    }
}

/// An RFC 5322 MIME message ready to be handed to [`SendEmail::send`].
///
/// The envelope `from`/`to` addresses drive the SMTP `MAIL FROM` and `RCPT TO`
/// commands and may legitimately differ from the `From:`/`To:` headers inside
/// `raw` — for example when implementing bounces, VERP, or BCC.
#[derive(Debug)]
pub struct EmailMessage(EmailMessageSys);

impl EmailMessage {
    /// Build a message from envelope addresses and a fully-formed RFC 5322
    /// MIME body.
    pub fn new(from: &str, to: &str, raw: &str) -> Result<Self> {
        Ok(Self(EmailMessageSys::new(from, to, raw)?))
    }
}
