use mail_builder::MessageBuilder as MimeBuilder;
use worker::*;

#[event(email)]
async fn email(message: ForwardableEmailMessage, _env: Env, _ctx: Context) -> Result<()> {
    let from = message.from();
    let to = message.to();

    let headers = Headers(message.headers());
    // The header value includes the surrounding angle brackets (`<id@host>`),
    // but `mail-builder` adds its own when serializing — pass the bare id.
    let in_reply_to = headers.get("message-id")?.map(|id| {
        id.trim()
            .trim_start_matches('<')
            .trim_end_matches('>')
            .to_string()
    });
    let original_subject = headers.get("subject")?.unwrap_or_default();
    let reply_subject = if original_subject.starts_with("Re:") {
        original_subject
    } else {
        format!("Re: {original_subject}")
    };

    // mail-builder's auto-generated `Date:` and `Message-ID:` headers rely on
    // `SystemTime::now()` and `gethostname`, neither of which work on
    // `wasm32-unknown-unknown`. https://github.com/stalwartlabs/mail-builder/pull/26
    let now_ms = Date::now().as_millis();
    let message_id = format!("{now_ms}@{}", domain_of(&to));

    let mut builder = MimeBuilder::new()
        .from(to.as_str())
        .to(from.as_str())
        .subject(reply_subject.as_str())
        .date((now_ms / 1000) as i64)
        .message_id(message_id)
        .text_body("message received");

    if let Some(id) = in_reply_to {
        builder = builder.in_reply_to(id.clone()).references(id);
    }

    let raw = builder
        .write_to_string()
        .map_err(|e| Error::RustError(e.to_string()))?;

    let reply = email::EmailMessage::new(&to, &from, &raw)?;
    message.reply(&reply).await?;
    Ok(())
}

fn domain_of(address: &str) -> &str {
    address
        .rsplit_once('@')
        .map(|(_, d)| d)
        .unwrap_or("example.com")
}
