use crate::SomeSharedData;
use worker::{Date, Email, EmailAddress, EmailMessage, Env, Request, Response, Result, SendEmail};

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
                Date::now().as_millis()
            ));
        }
        raw.push_str("\r\nhello from an integration test\r\n");
        raw
    }
}

fn build_structured(name: &str) -> Option<Result<Email>> {
    let builder = match name {
        "structured-ok" => Email::builder()
            .from(SENDER)
            .to(RECIPIENT)
            .subject("structured integration test")
            .text("hello from the structured path"),
        "structured-with-name" => Email::builder()
            .from(EmailAddress::with_name("Integration", SENDER))
            .to(RECIPIENT)
            .subject("structured integration test")
            .html("<p>hello from the structured path</p>"),
        "structured-disallowed-sender" => Email::builder()
            .from(BAD_SENDER)
            .to(RECIPIENT)
            .subject("structured integration test")
            .text("hello"),
        "structured-disallowed-recipient" => Email::builder()
            .from(SENDER)
            .to(BAD_RECIPIENT)
            .subject("structured integration test")
            .text("hello"),
        _ => return None,
    };
    Some(builder.build())
}

#[worker::send]
pub async fn handle_send_email(req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let url = req.url()?;
    let name = url
        .query_pairs()
        .find_map(|(k, v)| (k == "scenario").then(|| v.into_owned()))
        .unwrap_or_default();

    let sender = env.send_email("EMAIL")?;

    if let Some(scenario) = MimeScenario::for_name(&name) {
        return respond(dispatch_mime(&sender, &scenario).await);
    }

    if let Some(email_result) = build_structured(&name) {
        return respond(dispatch_structured(&sender, email_result).await);
    }

    Response::error(format!("unknown scenario: {name}"), 400)
}

async fn dispatch_mime(sender: &SendEmail, scenario: &MimeScenario) -> Result<String> {
    let message = EmailMessage::new(
        scenario.envelope_from,
        scenario.envelope_to,
        &scenario.raw(),
    )?;
    sender.send_mime(&message).await.map(|r| r.message_id)
}

async fn dispatch_structured(sender: &SendEmail, email: Result<Email>) -> Result<String> {
    let email = email?;
    sender.send(&email).await.map(|r| r.message_id)
}

fn respond(result: Result<String>) -> Result<Response> {
    Response::from_json(&serde_json::json!({
        "ok": result.is_ok(),
        "messageId": result.as_ref().ok(),
        "error": result.err().map(|e| e.to_string()),
    }))
}
