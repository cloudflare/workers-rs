use crate::{Env, SomeSharedData};
use js_sys::Uint8Array;
use ts_gen_worker::*;
use wasm_bindgen_futures::spawn_local;

const SENDER: &str = "allowed-sender@example.com";
const RECIPIENT: &str = "allowed-recipient@example.com";
const BAD_SENDER: &str = "evil@example.com";
const BAD_RECIPIENT: &str = "nope@example.com";
const MISMATCHED_FROM: &str = "mismatched@example.com";

struct MimeScenario {
    envelope_from: &'static str,
    envelope_to: &'static str,
    header_from: &'static str,
    include_message_id: bool,
}

impl MimeScenario {
    fn for_name(name: &str) -> Option<Self> {
        Some(match name {
            "mime-ok" => Self {
                envelope_from: SENDER,
                envelope_to: RECIPIENT,
                header_from: SENDER,
                include_message_id: true,
            },
            "mime-missing-message-id" => Self {
                envelope_from: SENDER,
                envelope_to: RECIPIENT,
                header_from: SENDER,
                include_message_id: false,
            },
            "mime-disallowed-sender" => Self {
                envelope_from: BAD_SENDER,
                envelope_to: RECIPIENT,
                header_from: BAD_SENDER,
                include_message_id: true,
            },
            "mime-disallowed-recipient" => Self {
                envelope_from: SENDER,
                envelope_to: BAD_RECIPIENT,
                header_from: SENDER,
                include_message_id: true,
            },
            "mime-from-mismatch" => Self {
                envelope_from: SENDER,
                envelope_to: RECIPIENT,
                header_from: MISMATCHED_FROM,
                include_message_id: true,
            },
            _ => return None,
        })
    }

    fn raw(&self) -> String {
        let mut raw = format!(
            "From: {}\r\n\
             To: {}\r\n\
             Subject: Integration test\r\n\
             Date: Thu, 01 Jan 1970 00:00:00 +0000\r\n",
            self.header_from, self.envelope_to
        );
        if self.include_message_id {
            raw.push_str(&format!(
                "Message-ID: <{}@example.com>\r\n",
                js_sys::Date::now() as u64
            ));
        }
        raw.push_str("\r\nhello from an integration test\r\n");
        raw
    }
}

/// Upstream's `EmailMessageBuilder` requires `from` and `subject`; one of
/// `to` / `cc` / `bcc` is required at runtime but each is optional in the type.
fn build_structured(name: &str) -> Option<EmailMessageBuilder> {
    let subject = "structured integration test";
    let builder = match name {
        "structured-ok" => EmailMessageBuilder::builder(SENDER, subject)
            .to(RECIPIENT)
            .text("hello from the structured path"),
        "structured-with-name" => {
            let address = EmailAddress::new("Integration", SENDER);
            EmailMessageBuilder::builder_with_email_address(&address, subject)
                .to(RECIPIENT)
                .html("<p>hello from the structured path</p>")
        }
        "structured-disallowed-sender" => EmailMessageBuilder::builder(BAD_SENDER, subject)
            .to(RECIPIENT)
            .text("hello"),
        "structured-disallowed-recipient" => EmailMessageBuilder::builder(SENDER, subject)
            .to(BAD_RECIPIENT)
            .text("hello"),
        "structured-with-attachment" => {
            let payload = b"hello attachment";
            let attachment = EmailAttachment::new_attachment_with_buffer_source(
                "hello.txt",
                "text/plain",
                &payload[..],
            );
            EmailMessageBuilder::builder(SENDER, subject)
                .to(RECIPIENT)
                .text("structured email with an attachment")
                .attachments(&[attachment])
        }
        _ => return None,
    };
    Some(builder.build())
}

pub async fn handle_send_email(req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let url = req.url_parsed()?;
    let name = url
        .query_pairs()
        .find_map(|(k, v)| (k == "scenario").then(|| v.into_owned()))
        .unwrap_or_default();

    let sender = env.email();

    if name == "mime-stream" {
        return respond(dispatch_mime_stream(&sender).await);
    }

    if let Some(scenario) = MimeScenario::for_name(&name) {
        return respond(dispatch_mime(&sender, &scenario).await);
    }

    if let Some(builder) = build_structured(&name) {
        return respond(dispatch_structured(&sender, builder).await);
    }

    Response::builder().error(format!("unknown scenario: {name}"), 400)
}

async fn dispatch_mime(sender: &SendEmail, scenario: &MimeScenario) -> Result<String> {
    let message =
        EmailMessage::new_with_str(scenario.envelope_from, scenario.envelope_to, scenario.raw())?;
    let result = sender.send(&message).await?;
    Ok(result.message_id())
}

/// `FixedLengthStream::wrap(stream, len)`: a one-chunk body written through the
/// stream's writable side.
async fn dispatch_mime_stream(sender: &SendEmail) -> Result<String> {
    let scenario = MimeScenario::for_name("mime-ok").expect("mime-ok scenario must exist");
    let raw = scenario.raw().into_bytes();
    let fixed = FixedLengthStream::new(raw.len() as f64)?;
    let writer = fixed.writable().get_writer()?;
    spawn_local(async move {
        let _ = writer.write_with_chunk(&Uint8Array::from(&raw[..])).await;
        let _ = writer.close().await;
    });
    let stream = fixed.readable();
    let message = EmailMessage::new(scenario.envelope_from, scenario.envelope_to, &stream)?;
    let result = sender.send(&message).await?;
    Ok(result.message_id())
}

async fn dispatch_structured(sender: &SendEmail, builder: EmailMessageBuilder) -> Result<String> {
    let result = sender.send_with_builder(&builder).await?;
    Ok(result.message_id())
}

fn respond(result: Result<String>) -> Result<Response> {
    Response::builder().from_json(&serde_json::json!({
        "ok": result.is_ok(),
        "messageId": result.as_ref().ok(),
        "error": result.err().map(|e| e.to_string()),
    }))
}
