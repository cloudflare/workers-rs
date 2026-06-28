use mail_builder::MessageBuilder as MimeBuilder;
use worker::*;

const SENDER: &str = "sender@example.com";
const RECIPIENT: &str = "recipient@example.com";

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let sender = env.send_email("EMAIL")?;

    let result = match req.path().as_str() {
        "/" => send_structured(&sender).await?,
        "/raw" => send_raw_mime(&sender).await?,
        // Don't dispatch on favicon / unknown paths — otherwise every browser
        // tab to localhost sends a real email in `wrangler dev`.
        _ => return Response::error("not found", 404),
    };

    Response::ok(format!("sent: {}", result.message_id()))
}

async fn send_structured(sender: &email::SendEmail) -> Result<email::EmailSendResult> {
    let from = email::EmailAddress::new("Sending email test", SENDER);
    let builder = email::SendEmailBuilder::builder_with_email_address_and_str(
        &from,
        RECIPIENT,
        "An email generated in a Worker",
    )
    .text("Congratulations, you just sent an email from a Worker.")
    .html("<p>Congratulations, you just sent an email from a Worker.</p>")
    .build();

    Ok(sender.send_with_builder(&builder).await?)
}

async fn send_raw_mime(sender: &email::SendEmail) -> Result<email::EmailSendResult> {
    // mail-builder's auto-generated `Date:` and `Message-ID:` headers rely on
    // `SystemTime::now()` and `gethostname`, neither of which work on
    // `wasm32-unknown-unknown`. https://github.com/stalwartlabs/mail-builder/pull/26
    let now_ms = Date::now().as_millis();
    let message_id = format!("{now_ms}@example.com");

    let raw = MimeBuilder::new()
        .from(("Sending email test", SENDER))
        .to(RECIPIENT)
        .subject("An email generated in a Worker")
        .date((now_ms / 1000) as i64)
        .message_id(message_id)
        .text_body("Congratulations, you just sent an email from a Worker.")
        .write_to_string()
        .map_err(|e| Error::RustError(e.to_string()))?;

    let message = email::EmailMessage::new(SENDER, RECIPIENT, &raw)?;
    Ok(sender.send(&message).await?)
}
