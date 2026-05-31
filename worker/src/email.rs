#[allow(dead_code)]
use crate::Context as ExecutionContext;
#[allow(dead_code)]
use crate::Env;
#[allow(dead_code)]
use ::web_sys::Event;
#[allow(dead_code)]
use ::web_sys::Headers;
#[allow(dead_code)]
use ::web_sys::ReadableStream;
#[allow(unused_imports)]
use js_sys::*;
#[allow(unused_imports)]
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = Event , extends = Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type ExtendableEvent;
    #[doc = " The **`ExtendableEvent.waitUntil()`** method tells the event dispatcher that work is ongoing."]
    #[doc = ""]
    #[doc = " [MDN Reference](https://developer.mozilla.org/docs/Web/API/ExtendableEvent/waitUntil)"]
    #[wasm_bindgen(method, js_name = "waitUntil")]
    pub fn wait_until(this: &ExtendableEvent, promise: &Promise);
    #[doc = " The **`ExtendableEvent.waitUntil()`** method tells the event dispatcher that work is ongoing."]
    #[doc = ""]
    #[doc = " [MDN Reference](https://developer.mozilla.org/docs/Web/API/ExtendableEvent/waitUntil)"]
    #[wasm_bindgen(method, catch, js_name = "waitUntil")]
    pub fn try_wait_until(this: &ExtendableEvent, promise: &Promise) -> Result<(), JsValue>;
}
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type EmailSendResult;
    #[doc = " The Email Message ID"]
    #[wasm_bindgen(method, getter, js_name = "messageId")]
    pub fn message_id(this: &EmailSendResult) -> String;
    #[wasm_bindgen(method, setter, js_name = "messageId")]
    pub fn set_message_id(this: &EmailSendResult, val: &str);
}
impl EmailSendResult {
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `message_id` - The Email Message ID"]
    pub fn new(message_id: &str) -> EmailSendResult {
        Self::builder(message_id).build()
    }
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `message_id` - The Email Message ID"]
    pub fn builder(message_id: &str) -> EmailSendResultBuilder {
        let inner: Self = JsCast::unchecked_into(js_sys::Object::new());
        inner.set_message_id(message_id);
        EmailSendResultBuilder { inner }
    }
}
pub struct EmailSendResultBuilder {
    inner: EmailSendResult,
}
impl EmailSendResultBuilder {
    pub fn build(self) -> EmailSendResult {
        self.inner
    }
}
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = email :: EmailMessage , extends = Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type ForwardableEmailMessage;
    #[doc = " Stream of the email message content."]
    #[wasm_bindgen(method, getter)]
    pub fn raw(this: &ForwardableEmailMessage) -> ReadableStream;
    #[doc = " An [Headers object](https://developer.mozilla.org/en-US/docs/Web/API/Headers)."]
    #[wasm_bindgen(method, getter)]
    pub fn headers(this: &ForwardableEmailMessage) -> Headers;
    #[doc = " Size of the email message content."]
    #[wasm_bindgen(method, getter, js_name = "rawSize")]
    pub fn raw_size(this: &ForwardableEmailMessage) -> f64;
    #[doc = " Reject this email message by returning a permanent SMTP error back to the connecting client including the given reason."]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `reason` - The reject reason."]
    #[doc = ""]
    #[doc = " ## Returns"]
    #[doc = ""]
    #[doc = " void"]
    #[wasm_bindgen(method, js_name = "setReject")]
    pub fn set_reject(this: &ForwardableEmailMessage, reason: &str);
    #[doc = " Reject this email message by returning a permanent SMTP error back to the connecting client including the given reason."]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `reason` - The reject reason."]
    #[doc = ""]
    #[doc = " ## Returns"]
    #[doc = ""]
    #[doc = " void"]
    #[wasm_bindgen(method, catch, js_name = "setReject")]
    pub fn try_set_reject(this: &ForwardableEmailMessage, reason: &str) -> Result<(), JsValue>;
    #[doc = " Forward this email message to a verified destination address of the account."]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `rcptTo` - Verified destination address."]
    #[doc = " * `headers` - A [Headers object](https://developer.mozilla.org/en-US/docs/Web/API/Headers)."]
    #[doc = ""]
    #[doc = " ## Returns"]
    #[doc = ""]
    #[doc = " A promise that resolves when the email message is forwarded."]
    #[wasm_bindgen(method, catch)]
    pub async fn forward(
        this: &ForwardableEmailMessage,
        rcpt_to: &str,
    ) -> Result<EmailSendResult, JsValue>;
    #[doc = " Forward this email message to a verified destination address of the account."]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `rcptTo` - Verified destination address."]
    #[doc = " * `headers` - A [Headers object](https://developer.mozilla.org/en-US/docs/Web/API/Headers)."]
    #[doc = ""]
    #[doc = " ## Returns"]
    #[doc = ""]
    #[doc = " A promise that resolves when the email message is forwarded."]
    #[wasm_bindgen(method, catch, js_name = "forward")]
    pub async fn forward_with_headers(
        this: &ForwardableEmailMessage,
        rcpt_to: &str,
        headers: &Headers,
    ) -> Result<EmailSendResult, JsValue>;
    #[doc = " Reply to the sender of this email message with a new EmailMessage object."]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `message` - The reply message."]
    #[doc = ""]
    #[doc = " ## Returns"]
    #[doc = ""]
    #[doc = " A promise that resolves when the email message is replied."]
    #[wasm_bindgen(method, catch)]
    pub async fn reply(
        this: &ForwardableEmailMessage,
        message: &email::EmailMessage,
    ) -> Result<EmailSendResult, JsValue>;
}
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type EmailAttachment;
    #[wasm_bindgen(method, getter)]
    pub fn content(this: &EmailAttachment) -> JsValue;
    #[wasm_bindgen(method, getter, js_name = "contentId")]
    pub fn content_id(this: &EmailAttachment) -> Option<JsValue>;
    #[wasm_bindgen(method, getter)]
    pub fn disposition(this: &EmailAttachment) -> JsValue;
    #[wasm_bindgen(method, getter)]
    pub fn filename(this: &EmailAttachment) -> String;
    #[wasm_bindgen(method, getter)]
    pub fn r#type(this: &EmailAttachment) -> String;
    #[wasm_bindgen(method, setter)]
    pub fn set_content(this: &EmailAttachment, val: &str);
    #[wasm_bindgen(method, setter, js_name = "content")]
    pub fn set_content_with_array_buffer(this: &EmailAttachment, val: &ArrayBuffer);
    #[wasm_bindgen(method, setter, js_name = "content")]
    pub fn set_content_with_js_value(this: &EmailAttachment, val: &Object);
    #[wasm_bindgen(method, setter, js_name = "contentId")]
    pub fn set_content_id(this: &EmailAttachment, val: &str);
    #[wasm_bindgen(method, setter, js_name = "contentId")]
    pub fn set_content_id_with_undefined(this: &EmailAttachment, val: &Undefined);
    #[wasm_bindgen(method, setter)]
    pub fn set_disposition(this: &EmailAttachment, val: &str);
    #[wasm_bindgen(method, setter, js_name = "disposition")]
    pub fn set_disposition_with_js_value(this: &EmailAttachment, val: &str);
    #[wasm_bindgen(method, setter)]
    pub fn set_filename(this: &EmailAttachment, val: &str);
    #[wasm_bindgen(method, setter)]
    pub fn set_type(this: &EmailAttachment, val: &str);
}
impl EmailAttachment {
    #[doc = " ## Inlined fields"]
    #[doc = ""]
    #[doc = " * `disposition: \"inline\"`"]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `content`"]
    #[doc = " * `filename`"]
    #[doc = " * `type`"]
    pub fn new_inline(content: &str, filename: &str, r#type: &str) -> EmailAttachment {
        Self::builder_inline(content, filename, r#type).build()
    }
    #[doc = " ## Inlined fields"]
    #[doc = ""]
    #[doc = " * `disposition: \"attachment\"`"]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `content`"]
    #[doc = " * `filename`"]
    #[doc = " * `type`"]
    pub fn new_attachment(content: &str, filename: &str, r#type: &str) -> EmailAttachment {
        Self::builder_attachment(content, filename, r#type).build()
    }
    #[doc = " ## Inlined fields"]
    #[doc = ""]
    #[doc = " * `disposition: \"inline\"`"]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `content`"]
    #[doc = " * `filename`"]
    #[doc = " * `type`"]
    pub fn new_inline_with_array_buffer(
        content: &ArrayBuffer,
        filename: &str,
        r#type: &str,
    ) -> EmailAttachment {
        Self::builder_inline_with_array_buffer(content, filename, r#type).build()
    }
    #[doc = " ## Inlined fields"]
    #[doc = ""]
    #[doc = " * `disposition: \"attachment\"`"]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `content`"]
    #[doc = " * `filename`"]
    #[doc = " * `type`"]
    pub fn new_attachment_with_array_buffer(
        content: &ArrayBuffer,
        filename: &str,
        r#type: &str,
    ) -> EmailAttachment {
        Self::builder_attachment_with_array_buffer(content, filename, r#type).build()
    }
    #[doc = " ## Inlined fields"]
    #[doc = ""]
    #[doc = " * `disposition: \"inline\"`"]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `content`"]
    #[doc = " * `filename`"]
    #[doc = " * `type`"]
    pub fn new_inline_with_js_value(
        content: &Object,
        filename: &str,
        r#type: &str,
    ) -> EmailAttachment {
        Self::builder_inline_with_js_value(content, filename, r#type).build()
    }
    #[doc = " ## Inlined fields"]
    #[doc = ""]
    #[doc = " * `disposition: \"attachment\"`"]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `content`"]
    #[doc = " * `filename`"]
    #[doc = " * `type`"]
    pub fn new_attachment_with_js_value(
        content: &Object,
        filename: &str,
        r#type: &str,
    ) -> EmailAttachment {
        Self::builder_attachment_with_js_value(content, filename, r#type).build()
    }
    #[doc = " ## Inlined fields"]
    #[doc = ""]
    #[doc = " * `disposition: \"inline\"`"]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `content`"]
    #[doc = " * `filename`"]
    #[doc = " * `type`"]
    pub fn builder_inline(content: &str, filename: &str, r#type: &str) -> EmailAttachmentBuilder {
        let inner: Self = JsCast::unchecked_into(js_sys::Object::new());
        inner.set_content(content);
        inner.set_disposition("inline");
        inner.set_filename(filename);
        inner.set_type(r#type);
        EmailAttachmentBuilder { inner }
    }
    #[doc = " ## Inlined fields"]
    #[doc = ""]
    #[doc = " * `disposition: \"attachment\"`"]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `content`"]
    #[doc = " * `filename`"]
    #[doc = " * `type`"]
    pub fn builder_attachment(
        content: &str,
        filename: &str,
        r#type: &str,
    ) -> EmailAttachmentBuilder {
        let inner: Self = JsCast::unchecked_into(js_sys::Object::new());
        inner.set_content(content);
        inner.set_disposition_with_js_value("attachment");
        inner.set_filename(filename);
        inner.set_type(r#type);
        EmailAttachmentBuilder { inner }
    }
    #[doc = " ## Inlined fields"]
    #[doc = ""]
    #[doc = " * `disposition: \"inline\"`"]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `content`"]
    #[doc = " * `filename`"]
    #[doc = " * `type`"]
    pub fn builder_inline_with_array_buffer(
        content: &ArrayBuffer,
        filename: &str,
        r#type: &str,
    ) -> EmailAttachmentBuilder {
        let inner: Self = JsCast::unchecked_into(js_sys::Object::new());
        inner.set_content_with_array_buffer(content);
        inner.set_disposition("inline");
        inner.set_filename(filename);
        inner.set_type(r#type);
        EmailAttachmentBuilder { inner }
    }
    #[doc = " ## Inlined fields"]
    #[doc = ""]
    #[doc = " * `disposition: \"attachment\"`"]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `content`"]
    #[doc = " * `filename`"]
    #[doc = " * `type`"]
    pub fn builder_attachment_with_array_buffer(
        content: &ArrayBuffer,
        filename: &str,
        r#type: &str,
    ) -> EmailAttachmentBuilder {
        let inner: Self = JsCast::unchecked_into(js_sys::Object::new());
        inner.set_content_with_array_buffer(content);
        inner.set_disposition_with_js_value("attachment");
        inner.set_filename(filename);
        inner.set_type(r#type);
        EmailAttachmentBuilder { inner }
    }
    #[doc = " ## Inlined fields"]
    #[doc = ""]
    #[doc = " * `disposition: \"inline\"`"]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `content`"]
    #[doc = " * `filename`"]
    #[doc = " * `type`"]
    pub fn builder_inline_with_js_value(
        content: &Object,
        filename: &str,
        r#type: &str,
    ) -> EmailAttachmentBuilder {
        let inner: Self = JsCast::unchecked_into(js_sys::Object::new());
        inner.set_content_with_js_value(content);
        inner.set_disposition("inline");
        inner.set_filename(filename);
        inner.set_type(r#type);
        EmailAttachmentBuilder { inner }
    }
    #[doc = " ## Inlined fields"]
    #[doc = ""]
    #[doc = " * `disposition: \"attachment\"`"]
    #[doc = ""]
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `content`"]
    #[doc = " * `filename`"]
    #[doc = " * `type`"]
    pub fn builder_attachment_with_js_value(
        content: &Object,
        filename: &str,
        r#type: &str,
    ) -> EmailAttachmentBuilder {
        let inner: Self = JsCast::unchecked_into(js_sys::Object::new());
        inner.set_content_with_js_value(content);
        inner.set_disposition_with_js_value("attachment");
        inner.set_filename(filename);
        inner.set_type(r#type);
        EmailAttachmentBuilder { inner }
    }
}
pub struct EmailAttachmentBuilder {
    inner: EmailAttachment,
}
impl EmailAttachmentBuilder {
    pub fn content_id(self, val: &str) -> Self {
        self.inner.set_content_id(val);
        self
    }
    pub fn content_id_with_undefined(self, val: &Undefined) -> Self {
        self.inner.set_content_id_with_undefined(val);
        self
    }
    pub fn build(self) -> EmailAttachment {
        self.inner
    }
}
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type EmailAddress;
    #[wasm_bindgen(method, getter)]
    pub fn name(this: &EmailAddress) -> String;
    #[wasm_bindgen(method, setter)]
    pub fn set_name(this: &EmailAddress, val: &str);
    #[wasm_bindgen(method, getter)]
    pub fn email(this: &EmailAddress) -> String;
    #[wasm_bindgen(method, setter)]
    pub fn set_email(this: &EmailAddress, val: &str);
}
impl EmailAddress {
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `name`"]
    #[doc = " * `email`"]
    pub fn new(name: &str, email: &str) -> EmailAddress {
        Self::builder(name, email).build()
    }
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `name`"]
    #[doc = " * `email`"]
    pub fn builder(name: &str, email: &str) -> EmailAddressBuilder {
        let inner: Self = JsCast::unchecked_into(js_sys::Object::new());
        inner.set_name(name);
        inner.set_email(email);
        EmailAddressBuilder { inner }
    }
}
pub struct EmailAddressBuilder {
    inner: EmailAddress,
}
impl EmailAddressBuilder {
    pub fn build(self) -> EmailAddress {
        self.inner
    }
}
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type SendEmail;
    #[wasm_bindgen(method, catch)]
    pub async fn send(
        this: &SendEmail,
        message: &email::EmailMessage,
    ) -> Result<EmailSendResult, JsValue>;
    #[wasm_bindgen(method, catch, js_name = "send")]
    pub async fn send_with_builder(
        this: &SendEmail,
        builder: &SendEmailBuilder,
    ) -> Result<EmailSendResult, JsValue>;
}
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type SendEmailBuilder;
    #[wasm_bindgen(method, getter)]
    pub fn from(this: &SendEmailBuilder) -> JsValue;
    #[wasm_bindgen(method, setter)]
    pub fn set_from(this: &SendEmailBuilder, val: &str);
    #[wasm_bindgen(method, setter, js_name = "from")]
    pub fn set_from_with_email_address(this: &SendEmailBuilder, val: &EmailAddress);
    #[wasm_bindgen(method, getter)]
    pub fn to(this: &SendEmailBuilder) -> JsValue;
    #[wasm_bindgen(method, setter)]
    pub fn set_to(this: &SendEmailBuilder, val: &str);
    #[wasm_bindgen(method, setter, js_name = "to")]
    pub fn set_to_with_array(this: &SendEmailBuilder, val: &Array<JsString>);
    #[wasm_bindgen(method, getter)]
    pub fn subject(this: &SendEmailBuilder) -> String;
    #[wasm_bindgen(method, setter)]
    pub fn set_subject(this: &SendEmailBuilder, val: &str);
    #[wasm_bindgen(method, getter, js_name = "replyTo")]
    pub fn reply_to(this: &SendEmailBuilder) -> Option<JsValue>;
    #[wasm_bindgen(method, setter, js_name = "replyTo")]
    pub fn set_reply_to(this: &SendEmailBuilder, val: &str);
    #[wasm_bindgen(method, setter, js_name = "replyTo")]
    pub fn set_reply_to_with_email_address(this: &SendEmailBuilder, val: &EmailAddress);
    #[wasm_bindgen(method, getter)]
    pub fn cc(this: &SendEmailBuilder) -> Option<JsValue>;
    #[wasm_bindgen(method, setter)]
    pub fn set_cc(this: &SendEmailBuilder, val: &str);
    #[wasm_bindgen(method, setter, js_name = "cc")]
    pub fn set_cc_with_array(this: &SendEmailBuilder, val: &Array<JsString>);
    #[wasm_bindgen(method, getter)]
    pub fn bcc(this: &SendEmailBuilder) -> Option<JsValue>;
    #[wasm_bindgen(method, setter)]
    pub fn set_bcc(this: &SendEmailBuilder, val: &str);
    #[wasm_bindgen(method, setter, js_name = "bcc")]
    pub fn set_bcc_with_array(this: &SendEmailBuilder, val: &Array<JsString>);
    #[wasm_bindgen(method, getter)]
    pub fn headers(this: &SendEmailBuilder) -> Option<Object<JsString>>;
    #[wasm_bindgen(method, setter)]
    pub fn set_headers(this: &SendEmailBuilder, val: &Object<JsString>);
    #[wasm_bindgen(method, getter)]
    pub fn text(this: &SendEmailBuilder) -> Option<String>;
    #[wasm_bindgen(method, setter)]
    pub fn set_text(this: &SendEmailBuilder, val: &str);
    #[wasm_bindgen(method, getter)]
    pub fn html(this: &SendEmailBuilder) -> Option<String>;
    #[wasm_bindgen(method, setter)]
    pub fn set_html(this: &SendEmailBuilder, val: &str);
    #[wasm_bindgen(method, getter)]
    pub fn attachments(this: &SendEmailBuilder) -> Option<Array<EmailAttachment>>;
    #[wasm_bindgen(method, setter)]
    pub fn set_attachments(this: &SendEmailBuilder, val: &Array<EmailAttachment>);
}
impl SendEmailBuilder {
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `from`"]
    #[doc = " * `to`"]
    #[doc = " * `subject`"]
    pub fn new(from: &str, to: &str, subject: &str) -> SendEmailBuilder {
        Self::builder(from, to, subject).build()
    }
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `from`"]
    #[doc = " * `to`"]
    #[doc = " * `subject`"]
    pub fn new_with_str_and_array(
        from: &str,
        to: &Array<JsString>,
        subject: &str,
    ) -> SendEmailBuilder {
        Self::builder_with_str_and_array(from, to, subject).build()
    }
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `from`"]
    #[doc = " * `to`"]
    #[doc = " * `subject`"]
    pub fn new_with_email_address_and_str(
        from: &EmailAddress,
        to: &str,
        subject: &str,
    ) -> SendEmailBuilder {
        Self::builder_with_email_address_and_str(from, to, subject).build()
    }
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `from`"]
    #[doc = " * `to`"]
    #[doc = " * `subject`"]
    pub fn new_with_email_address_and_array(
        from: &EmailAddress,
        to: &Array<JsString>,
        subject: &str,
    ) -> SendEmailBuilder {
        Self::builder_with_email_address_and_array(from, to, subject).build()
    }
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `from`"]
    #[doc = " * `to`"]
    #[doc = " * `subject`"]
    pub fn builder(from: &str, to: &str, subject: &str) -> SendEmailBuilderBuilder {
        let inner: Self = JsCast::unchecked_into(js_sys::Object::new());
        inner.set_from(from);
        inner.set_to(to);
        inner.set_subject(subject);
        SendEmailBuilderBuilder { inner }
    }
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `from`"]
    #[doc = " * `to`"]
    #[doc = " * `subject`"]
    pub fn builder_with_str_and_array(
        from: &str,
        to: &Array<JsString>,
        subject: &str,
    ) -> SendEmailBuilderBuilder {
        let inner: Self = JsCast::unchecked_into(js_sys::Object::new());
        inner.set_from(from);
        inner.set_to_with_array(to);
        inner.set_subject(subject);
        SendEmailBuilderBuilder { inner }
    }
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `from`"]
    #[doc = " * `to`"]
    #[doc = " * `subject`"]
    pub fn builder_with_email_address_and_str(
        from: &EmailAddress,
        to: &str,
        subject: &str,
    ) -> SendEmailBuilderBuilder {
        let inner: Self = JsCast::unchecked_into(js_sys::Object::new());
        inner.set_from_with_email_address(from);
        inner.set_to(to);
        inner.set_subject(subject);
        SendEmailBuilderBuilder { inner }
    }
    #[doc = " ## Arguments"]
    #[doc = ""]
    #[doc = " * `from`"]
    #[doc = " * `to`"]
    #[doc = " * `subject`"]
    pub fn builder_with_email_address_and_array(
        from: &EmailAddress,
        to: &Array<JsString>,
        subject: &str,
    ) -> SendEmailBuilderBuilder {
        let inner: Self = JsCast::unchecked_into(js_sys::Object::new());
        inner.set_from_with_email_address(from);
        inner.set_to_with_array(to);
        inner.set_subject(subject);
        SendEmailBuilderBuilder { inner }
    }
}
pub struct SendEmailBuilderBuilder {
    inner: SendEmailBuilder,
}
impl SendEmailBuilderBuilder {
    pub fn reply_to(self, val: &str) -> Self {
        self.inner.set_reply_to(val);
        self
    }
    pub fn reply_to_with_email_address(self, val: &EmailAddress) -> Self {
        self.inner.set_reply_to_with_email_address(val);
        self
    }
    pub fn cc(self, val: &str) -> Self {
        self.inner.set_cc(val);
        self
    }
    pub fn cc_with_array(self, val: &Array<JsString>) -> Self {
        self.inner.set_cc_with_array(val);
        self
    }
    pub fn bcc(self, val: &str) -> Self {
        self.inner.set_bcc(val);
        self
    }
    pub fn bcc_with_array(self, val: &Array<JsString>) -> Self {
        self.inner.set_bcc_with_array(val);
        self
    }
    pub fn headers(self, val: &Object<JsString>) -> Self {
        self.inner.set_headers(val);
        self
    }
    pub fn text(self, val: &str) -> Self {
        self.inner.set_text(val);
        self
    }
    pub fn html(self, val: &str) -> Self {
        self.inner.set_html(val);
        self
    }
    pub fn attachments(self, val: &Array<EmailAttachment>) -> Self {
        self.inner.set_attachments(val);
        self
    }
    pub fn build(self) -> SendEmailBuilder {
        self.inner
    }
}
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = ExtendableEvent , extends = Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type EmailEvent;
    #[wasm_bindgen(method, getter)]
    pub fn message(this: &EmailEvent) -> ForwardableEmailMessage;
}
#[allow(dead_code)]
pub type EmailExportedHandler =
    Function<fn(ForwardableEmailMessage, Env, ExecutionContext) -> JsOption<Promise<Undefined>>>;
pub mod email {
    use super::*;
    use js_sys::*;
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen(module = "cloudflare:email")]
    extern "C" {
        # [wasm_bindgen (extends = Object)]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub type EmailMessage;
        #[wasm_bindgen(constructor, catch)]
        pub fn new(from: &str, to: &str, raw: &str) -> Result<EmailMessage, JsValue>;
        #[wasm_bindgen(constructor, catch, js_name = "EmailMessage")]
        pub fn new_with_readable_stream(
            from: &str,
            to: &str,
            raw: &ReadableStream,
        ) -> Result<EmailMessage, JsValue>;
        #[doc = " Envelope From attribute of the email message."]
        #[wasm_bindgen(method, getter)]
        pub fn from(this: &EmailMessage) -> String;
        #[doc = " Envelope To attribute of the email message."]
        #[wasm_bindgen(method, getter)]
        pub fn to(this: &EmailMessage) -> String;
    }
}
