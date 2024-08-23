use core::str;

use worker::*;

#[event(email)]
async fn email(message: ForwardableEmailMessage, _env: Env, _ctx: Context) -> Result<()> {
    console_error_panic_hook::set_once();

    let allow_list = ["another@example.com", "coworker@example.com"];
    let from = message.from_envelope();

    let raw: Vec<u8> = message.raw_bytes().await?;
    let raw = str::from_utf8(&raw)?;
    console_log!("Raw email: {}", raw);

    if allow_list.contains(&from.as_str()) {
        message.forward("mailbox@anotherexample.com", None).await?;
    } else {
        message.set_reject("Address not allowed")?;
    }
    Ok(())
}
