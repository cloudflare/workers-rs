use mail_builder::MessageBuilder;
use worker::*;

const SENDER: &str = "sender@example.com";
const RECIPIENT: &str = "recipient@example.com";

#[event(fetch)]
async fn fetch(_req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // mail-builder's auto-generated `Date:` and `Message-ID:` headers rely on
    // `SystemTime::now()` and `gethostname`, neither of which work on
    // `wasm32-unknown-unknown`. https://github.com/stalwartlabs/mail-builder/pull/26
    let now_ms = Date::now().as_millis();
    let message_id = format!("{now_ms}@example.com");

    let raw = MessageBuilder::new()
        .from(("Sending email test", SENDER))
        .to(RECIPIENT)
        .subject("An email generated in a Worker")
        .date((now_ms / 1000) as i64)
        .message_id(message_id)
        .text_body("Congratulations, you just sent an email from a Worker.")
        .write_to_string()
        .map_err(|e| Error::RustError(e.to_string()))?;

    let email = EmailMessage::new(SENDER, RECIPIENT, &raw)?;
    env.send_email("EMAIL")?.send(&email).await?;

    Response::ok("sent")
}
