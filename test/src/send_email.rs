use crate::SomeSharedData;
use worker::{Date, EmailMessage, Env, Request, Response, Result};

const SENDER: &str = "allowed-sender@example.com";
const RECIPIENT: &str = "allowed-recipient@example.com";
const BAD_SENDER: &str = "evil@example.com";
const BAD_RECIPIENT: &str = "nope@example.com";
const MISMATCHED_FROM: &str = "mismatched@example.com";

struct Scenario {
    envelope_from: &'static str,
    envelope_to: &'static str,
    header_from: &'static str,
    include_message_id: bool,
}

impl Scenario {
    fn for_name(name: &str) -> Option<Self> {
        Some(match name {
            "ok" => Self {
                envelope_from: SENDER,
                envelope_to: RECIPIENT,
                header_from: SENDER,
                include_message_id: true,
            },
            "missing-message-id" => Self {
                envelope_from: SENDER,
                envelope_to: RECIPIENT,
                header_from: SENDER,
                include_message_id: false,
            },
            "disallowed-sender" => Self {
                envelope_from: BAD_SENDER,
                envelope_to: RECIPIENT,
                header_from: BAD_SENDER,
                include_message_id: true,
            },
            "disallowed-recipient" => Self {
                envelope_from: SENDER,
                envelope_to: BAD_RECIPIENT,
                header_from: SENDER,
                include_message_id: true,
            },
            "from-mismatch" => Self {
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

#[worker::send]
pub async fn handle_send_email(req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let url = req.url()?;
    let name = url
        .query_pairs()
        .find_map(|(k, v)| (k == "scenario").then(|| v.into_owned()))
        .unwrap_or_default();

    let Some(scenario) = Scenario::for_name(&name) else {
        return Response::error(format!("unknown scenario: {name}"), 400);
    };

    let message = EmailMessage::new(
        scenario.envelope_from,
        scenario.envelope_to,
        &scenario.raw(),
    )?;
    let result = env.send_email("EMAIL")?.send(&message).await;
    Response::from_json(&serde_json::json!({
        "ok": result.is_ok(),
        "error": result.err().map(|e| e.to_string()),
    }))
}
