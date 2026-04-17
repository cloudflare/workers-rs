use serde::{
    ser::{SerializeMap, SerializeStruct},
    Deserialize, Serialize, Serializer,
};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use worker_sys::{EmailMessage as EmailMessageSys, SendEmail as SendEmailSys};

use crate::{error::Error, send::SendFuture, EnvBinding, Result};

/// A binding to the [Cloudflare Email Sending] service, declared under
/// `[[send_email]]` in `wrangler.toml` and retrieved via
/// [`Env::send_email`](crate::Env::send_email).
///
/// Two send paths are supported, mirroring the JS `send()` overloads:
///
/// * [`SendEmail::send`] — hand it a structured [`Email`]; the runtime builds
///   the MIME body and dispatches. This is the common path.
/// * [`SendEmail::send_mime`] — hand it a prebuilt [`EmailMessage`] (a
///   fully-formed RFC 5322 MIME blob plus envelope addresses) for cases that
///   need precise MIME control.
///
/// Both return an [`EmailSendResult`] containing the runtime-assigned message
/// id (useful for logging and correlation).
///
/// [Cloudflare Email Sending]: https://developers.cloudflare.com/email-service/api/send-emails/workers-api/
///
/// ```ignore
/// use worker::*;
///
/// #[event(fetch)]
/// async fn fetch(_req: Request, env: Env, _ctx: Context) -> Result<Response> {
///     let email = Email::builder()
///         .from(("Acme", "noreply@acme.test"))
///         .to("user@example.com")
///         .subject("Welcome")
///         .text("Thanks for signing up.")
///         .build()?;
///     let result = env.send_email("EMAIL")?.send(&email).await?;
///     Response::ok(result.message_id)
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
    /// Dispatch a structured [`Email`]; the runtime composes the MIME body.
    pub async fn send(&self, email: &Email) -> Result<EmailSendResult> {
        // `serialize_maps_as_objects(true)` gives plain JS objects for the
        // `headers` map; default `serialize_bytes` behavior produces a
        // `Uint8Array` for binary attachment content.
        let ser = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
        let payload = email.serialize(&ser).map_err(JsValue::from)?;
        self.send_js(&payload).await
    }

    /// Dispatch a prebuilt [`EmailMessage`] containing a fully-formed RFC 5322
    /// MIME body. Prefer [`send`](Self::send) for typical use — this path is
    /// for cases where you need precise control over the MIME structure
    /// (custom headers, DKIM passthrough, VERP bounces, etc.).
    pub async fn send_mime(&self, message: &EmailMessage) -> Result<EmailSendResult> {
        self.send_js(message.0.as_ref()).await
    }

    async fn send_js(&self, payload: &JsValue) -> Result<EmailSendResult> {
        let promise = self.0.send(payload)?;
        let value = SendFuture::new(JsFuture::from(promise)).await?;
        // Miniflare's `send_email` binding resolves to `undefined`; real
        // workerd resolves to `{ messageId }`. Tolerate both so local dev
        // with `wrangler dev` doesn't throw on deserialize.
        if value.is_undefined() || value.is_null() {
            return Ok(EmailSendResult::default());
        }
        Ok(serde_wasm_bindgen::from_value(value)?)
    }
}

/// Return value of a successful send.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailSendResult {
    /// The runtime-assigned message id (also exposed as the `Message-ID`
    /// header on the delivered message).
    ///
    /// Empty under `miniflare` (used by `wrangler dev`), which resolves the
    /// binding with no value; real workerd populates it.
    pub message_id: String,
}

/// An RFC 5322 MIME message ready to be handed to [`SendEmail::send_mime`].
///
/// The envelope `from`/`to` addresses drive the SMTP `MAIL FROM` and `RCPT TO`
/// commands and may legitimately differ from the `From:`/`To:` headers inside
/// `raw` — for example when implementing bounces, VERP, or BCC. For everyday
/// use where you don't care about that distinction, prefer [`Email`] and
/// [`SendEmail::send`].
#[derive(Debug)]
pub struct EmailMessage(EmailMessageSys);

impl EmailMessage {
    /// Build a message from envelope addresses and a fully-formed RFC 5322
    /// MIME body.
    pub fn new(from: &str, to: &str, raw: &str) -> Result<Self> {
        Ok(Self(EmailMessageSys::new(from, to, raw)?))
    }
}

/// An email address, optionally annotated with a display name.
///
/// Accepts several convenient forms via [`From`]: `"a@b.com"`,
/// `("Name", "a@b.com")`, or construct explicitly via
/// [`EmailAddress::new`]/[`EmailAddress::with_name`].
#[derive(Debug, Clone)]
pub struct EmailAddress {
    pub email: String,
    pub name: Option<String>,
}

impl EmailAddress {
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: None,
        }
    }

    pub fn with_name(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: Some(name.into()),
        }
    }
}

// Emits a bare string when there's no display name, and `{ email, name }`
// otherwise — matches the shape the runtime expects.
impl Serialize for EmailAddress {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match &self.name {
            None => serializer.serialize_str(&self.email),
            Some(name) => {
                let mut st = serializer.serialize_struct("EmailAddress", 2)?;
                st.serialize_field("email", &self.email)?;
                st.serialize_field("name", name)?;
                st.end()
            }
        }
    }
}

impl From<&str> for EmailAddress {
    fn from(email: &str) -> Self {
        Self::new(email)
    }
}

impl From<String> for EmailAddress {
    fn from(email: String) -> Self {
        Self::new(email)
    }
}

// (name, email) — matches the "Display Name <addr>" reading order.
impl<N: Into<String>, E: Into<String>> From<(N, E)> for EmailAddress {
    fn from((name, email): (N, E)) -> Self {
        Self::with_name(name, email)
    }
}

/// Content for an [`EmailAttachment`] — either a pre-encoded base64 string or
/// raw bytes (serialized as a `Uint8Array`).
#[derive(Debug, Clone)]
pub enum AttachmentContent {
    Base64(String),
    Binary(Vec<u8>),
}

impl Serialize for AttachmentContent {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            AttachmentContent::Base64(s) => serializer.serialize_str(s),
            // With the non-`json_compatible` serde_wasm_bindgen serializer this
            // produces a `Uint8Array`, which is what the runtime expects.
            AttachmentContent::Binary(bytes) => serializer.serialize_bytes(bytes),
        }
    }
}

impl From<String> for AttachmentContent {
    fn from(s: String) -> Self {
        AttachmentContent::Base64(s)
    }
}

impl From<&str> for AttachmentContent {
    fn from(s: &str) -> Self {
        AttachmentContent::Base64(s.to_owned())
    }
}

impl From<Vec<u8>> for AttachmentContent {
    fn from(bytes: Vec<u8>) -> Self {
        AttachmentContent::Binary(bytes)
    }
}

impl From<&[u8]> for AttachmentContent {
    fn from(bytes: &[u8]) -> Self {
        AttachmentContent::Binary(bytes.to_vec())
    }
}

/// A file attachment for an [`Email`].
///
/// Use [`EmailAttachment::attachment`] for a regular downloadable attachment,
/// or [`EmailAttachment::inline`] for an image referenced by `cid:` in the
/// HTML body (requires a `content_id`).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "disposition", rename_all = "lowercase")]
pub enum EmailAttachment {
    Attachment {
        filename: String,
        #[serde(rename = "type")]
        content_type: String,
        content: AttachmentContent,
    },
    Inline {
        #[serde(rename = "contentId")]
        content_id: String,
        filename: String,
        #[serde(rename = "type")]
        content_type: String,
        content: AttachmentContent,
    },
}

impl EmailAttachment {
    pub fn attachment(
        filename: impl Into<String>,
        content_type: impl Into<String>,
        content: impl Into<AttachmentContent>,
    ) -> Self {
        EmailAttachment::Attachment {
            filename: filename.into(),
            content_type: content_type.into(),
            content: content.into(),
        }
    }

    pub fn inline(
        content_id: impl Into<String>,
        filename: impl Into<String>,
        content_type: impl Into<String>,
        content: impl Into<AttachmentContent>,
    ) -> Self {
        EmailAttachment::Inline {
            content_id: content_id.into(),
            filename: filename.into(),
            content_type: content_type.into(),
            content: content.into(),
        }
    }
}

/// A structured email message, dispatched via [`SendEmail::send`].
///
/// Build with [`Email::builder`].
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Email {
    from: EmailAddress,
    to: Vec<String>,
    subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    cc: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    bcc: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_to: Option<EmailAddress>,
    // `Vec<(String, String)>` preserves insertion order (duplicate header
    // names are meaningful in RFC 5322) but serializes as a plain JS object.
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "serialize_headers"
    )]
    headers: Vec<(String, String)>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    attachments: Vec<EmailAttachment>,
}

impl Email {
    #[must_use]
    pub fn builder() -> EmailBuilder {
        EmailBuilder::default()
    }
}

fn serialize_headers<S: Serializer>(
    headers: &[(String, String)],
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    let mut map = serializer.serialize_map(Some(headers.len()))?;
    for (k, v) in headers {
        map.serialize_entry(k, v)?;
    }
    map.end()
}

/// Fluent builder for [`Email`]. See [`Email::builder`].
#[derive(Debug, Clone, Default)]
pub struct EmailBuilder {
    from: Option<EmailAddress>,
    to: Vec<String>,
    subject: Option<String>,
    html: Option<String>,
    text: Option<String>,
    cc: Vec<String>,
    bcc: Vec<String>,
    reply_to: Option<EmailAddress>,
    headers: Vec<(String, String)>,
    attachments: Vec<EmailAttachment>,
}

impl EmailBuilder {
    #[must_use]
    pub fn from(mut self, from: impl Into<EmailAddress>) -> Self {
        self.from = Some(from.into());
        self
    }

    /// Add a single recipient. Can be called multiple times (the runtime
    /// accepts up to 50 recipients per send).
    #[must_use]
    pub fn to(mut self, recipient: impl Into<String>) -> Self {
        self.to.push(recipient.into());
        self
    }

    /// Replace the recipient list with the given iterator.
    #[must_use]
    pub fn to_all<I, S>(mut self, recipients: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.to = recipients.into_iter().map(Into::into).collect();
        self
    }

    #[must_use]
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    #[must_use]
    pub fn html(mut self, html: impl Into<String>) -> Self {
        self.html = Some(html.into());
        self
    }

    #[must_use]
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    #[must_use]
    pub fn cc(mut self, recipient: impl Into<String>) -> Self {
        self.cc.push(recipient.into());
        self
    }

    #[must_use]
    pub fn bcc(mut self, recipient: impl Into<String>) -> Self {
        self.bcc.push(recipient.into());
        self
    }

    #[must_use]
    pub fn reply_to(mut self, reply_to: impl Into<EmailAddress>) -> Self {
        self.reply_to = Some(reply_to.into());
        self
    }

    #[must_use]
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    #[must_use]
    pub fn attachment(mut self, attachment: EmailAttachment) -> Self {
        self.attachments.push(attachment);
        self
    }

    /// Finalize the builder.
    ///
    /// Returns an error if `from`, `to`, or `subject` are missing. Body
    /// validation (need at least one of `html` / `text`) is deferred to the
    /// runtime, which produces a more specific error.
    pub fn build(self) -> Result<Email> {
        let from = self
            .from
            .ok_or_else(|| Error::RustError("EmailBuilder::build: missing `from`".into()))?;
        if self.to.is_empty() {
            return Err(Error::RustError(
                "EmailBuilder::build: missing `to`".into(),
            ));
        }
        let subject = self
            .subject
            .ok_or_else(|| Error::RustError("EmailBuilder::build: missing `subject`".into()))?;
        Ok(Email {
            from,
            to: self.to,
            subject,
            html: self.html,
            text: self.text,
            cc: self.cc,
            bcc: self.bcc,
            reply_to: self.reply_to,
            headers: self.headers,
            attachments: self.attachments,
        })
    }
}
