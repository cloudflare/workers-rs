# Sending email from Cloudflare Workers

Example of using `worker::SendEmail` to send a message through a `[[send_email]]` binding.

Two routes:

* `GET /` — the structured path. Set fields like `from`, `to`, `subject`, and `text`/`html` on [`Message::builder`](https://docs.rs/worker/latest/worker/struct.MessageBuilder.html), and the runtime assembles the MIME body for you.
* `GET /raw` — the raw MIME path. Build the body yourself with [`mail-builder`](https://crates.io/crates/mail-builder) and hand it to [`EmailMessage`](https://docs.rs/worker/latest/worker/struct.EmailMessage.html) as-is. Reach for this when you need control over the MIME — custom headers, DKIM passthrough, VERP bounces, that sort of thing.

## Local development

`wrangler dev --local` won't actually send anything. As the [Cloudflare docs](https://developers.cloudflare.com/email-routing/email-workers/local-development/) explain, outbound messages get written to a local `.eml` file. Wrangler prints the path so you can open it and check the raw message.

```bash
npm install
npm run dev
# then, in another shell:
curl http://localhost:8787/        # structured
curl http://localhost:8787/raw     # raw MIME
```

## Deploying

Verify the sender and recipient addresses first (see the [Cloudflare email API docs](https://developers.cloudflare.com/email-service/api/send-emails/workers-api/)), then:

```bash
npm run deploy
```
