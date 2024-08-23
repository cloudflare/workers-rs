use futures_util::TryStreamExt;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use worker_sys::EmailMessage as EmailMessageExt;
use worker_sys::ForwardableEmailMessage as ForwardableEmailMessageExt;
use worker_sys::SendEmail as SendEmailExt;

use crate::ByteStream;
use crate::EnvBinding;
use crate::Error;
use crate::Headers;

pub struct EmailMessage(EmailMessageExt);

unsafe impl Send for EmailMessage {}
unsafe impl Sync for EmailMessage {}

impl JsCast for EmailMessage {
    fn instanceof(val: &JsValue) -> bool {
        val.is_instance_of::<EmailMessageExt>()
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        EmailMessage(val.unchecked_into())
    }

    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}

impl AsRef<JsValue> for EmailMessage {
    fn as_ref(&self) -> &JsValue {
        &self.0
    }
}

impl From<JsValue> for EmailMessage {
    fn from(val: JsValue) -> Self {
        EmailMessage(val.unchecked_into())
    }
}

impl From<EmailMessage> for JsValue {
    fn from(sec: EmailMessage) -> Self {
        sec.0.into()
    }
}

impl EmailMessage {
    pub fn new(from: &str, to: &str, raw: &str) -> Result<Self, JsValue> {
        Ok(EmailMessage(EmailMessageExt::new(from, to, raw)?))
    }

    // method from is renamed compared to the JS version, because `from` conflicts with the From trait
    // to was also renamed for consistency
    pub fn from_envelope(&self) -> String {
        self.0.from()
    }

    pub fn to_envelope(&self) -> String {
        self.0.to()
    }
}

pub struct ForwardableEmailMessage(ForwardableEmailMessageExt);

unsafe impl Send for ForwardableEmailMessage {}
unsafe impl Sync for ForwardableEmailMessage {}

impl JsCast for ForwardableEmailMessage {
    fn instanceof(val: &JsValue) -> bool {
        val.is_instance_of::<ForwardableEmailMessageExt>()
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        ForwardableEmailMessage(val.unchecked_into())
    }

    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}

impl AsRef<JsValue> for ForwardableEmailMessage {
    fn as_ref(&self) -> &JsValue {
        &self.0
    }
}

impl From<JsValue> for ForwardableEmailMessage {
    fn from(val: JsValue) -> Self {
        ForwardableEmailMessage(val.unchecked_into())
    }
}

impl From<ForwardableEmailMessage> for JsValue {
    fn from(sec: ForwardableEmailMessage) -> Self {
        sec.0.into()
    }
}

impl From<ForwardableEmailMessageExt> for ForwardableEmailMessage {
    fn from(value: ForwardableEmailMessageExt) -> Self {
        ForwardableEmailMessage(value)
    }
}

impl ForwardableEmailMessage {
    // method from is renamed compared to the JS version, because `from` conflicts with the From trait
    // to was also renamed for consistency
    pub fn from_envelope(&self) -> String {
        self.0.from()
    }

    pub fn to_envelope(&self) -> String {
        self.0.to()
    }

    pub fn raw(&self) -> Result<ByteStream, JsValue> {
        Ok(ByteStream {
            inner: wasm_streams::ReadableStream::from_raw(self.0.raw().dyn_into()?).into_stream(),
        })
    }

    pub async fn raw_bytes(&self) -> Result<Vec<u8>, Error> {
        self.raw()?
            .try_fold(Vec::new(), |mut bytes, mut chunk| async move {
                bytes.append(&mut chunk);
                Ok(bytes)
            })
            .await
    }

    pub fn raw_size(&self) -> u32 {
        self.0.raw_size()
    }

    pub fn set_reject(&self, reason: &str) -> Result<(), JsValue> {
        self.0.set_reject(reason)
    }

    pub async fn forward(&self, rcpt_to: &str, headers: Option<Headers>) -> Result<(), JsValue> {
        JsFuture::from(self.0.forward(rcpt_to, headers.unwrap_or_default().0)?).await?;
        Ok(())
    }

    pub async fn reply(&self, message: EmailMessage) -> Result<(), JsValue> {
        JsFuture::from(self.0.reply(message.0)?).await?;
        Ok(())
    }
}

pub struct SendEmail(SendEmailExt);

unsafe impl Send for SendEmail {}
unsafe impl Sync for SendEmail {}

impl JsCast for SendEmail {
    fn instanceof(val: &JsValue) -> bool {
        val.is_instance_of::<SendEmailExt>()
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        SendEmail(val.unchecked_into())
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

impl From<JsValue> for SendEmail {
    fn from(val: JsValue) -> Self {
        SendEmail(val.unchecked_into())
    }
}

impl From<SendEmail> for JsValue {
    fn from(sec: SendEmail) -> Self {
        sec.0.into()
    }
}

impl From<SendEmailExt> for SendEmail {
    fn from(value: SendEmailExt) -> Self {
        SendEmail(value)
    }
}

impl EnvBinding for SendEmail {
    const TYPE_NAME: &'static str = "SendEmail";
}

impl SendEmail {
    pub async fn send(&self, email: EmailMessage) -> Result<(), JsValue> {
        JsFuture::from(self.0.send(email.0)?).await?;
        Ok(())
    }
}
