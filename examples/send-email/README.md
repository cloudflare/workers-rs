# Sending Email from Cloudflare Workers

Demonstration of using `worker::SendEmail` to dispatch an outbound message
through a `[[send_email]]` binding.

Two paths are shown:

* `GET /` — the **structured** path, using
  [`Message::builder`](https://docs.rs/worker/latest/worker/struct.MessageBuilder.html).
  The runtime composes the MIME body from the fields you set (`from`, `to`,
  `subject`, `text`/`html`, attachments, etc.).
* `GET /raw` — the **raw MIME** path, using
  [`EmailMessage`](https://docs.rs/worker/latest/worker/struct.EmailMessage.html).
  The MIME body is built locally with
  [`mail-builder`](https://crates.io/crates/mail-builder) and handed verbatim
  to the binding. Use this when you need precise control over the MIME
  structure (custom headers, DKIM passthrough, VERP bounces, etc.).

## Local development

Running `wrangler dev --local` does **not** actually deliver the email. Per
the [Cloudflare docs](https://developers.cloudflare.com/email-routing/email-workers/local-development/),
outbound messages are simulated by writing each one to a local `.eml` file —
the path is printed in the terminal so you can inspect the raw message.

```bash
npm install
npm run dev
# then, in another shell:
curl http://localhost:8787/        # structured
curl http://localhost:8787/raw     # raw MIME
```

## Deploying

Before deploying, verify the sender and recipient addresses as documented at
<https://developers.cloudflare.com/email-service/api/send-emails/workers-api/>,
then:

```bash
npm run deploy
```
